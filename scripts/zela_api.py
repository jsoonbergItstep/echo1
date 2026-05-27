import hashlib
import json
import logging
import httpx
import time
from pathlib import Path
import os


def get_zela_vars() -> tuple[str, str, str]:
	key_id = os.getenv("ZELA_CLIENT_ID")
	key_secret = os.getenv("ZELA_PRIVATE_KEY")
	token_url = os.getenv("ZELA_TOKEN_URL").rstrip("/")
	executor_url = os.getenv("ZELA_EXECUTOR_URL").rstrip("/")
	core_url = os.getenv("ZELA_CORE_URL").rstrip("/")

	assert all([key_id, key_secret, token_url, executor_url, core_url]), (
		"One or more required environment variables are missing"
	)

	token = get_token(
		key_id,
		key_secret,
		token_url,
		verify_ssl=False,
	)
	logging.debug(f"Obtained token: '{token}'")

	return token, executor_url, core_url


def get_token(
	key_id: str, key_secret: str, oath_url: str, verify_ssl: bool = True
) -> str:
	"""Get OAuth2 access token from Zela auth server."""
	token_resp = httpx.post(
		oath_url,
		auth=(key_id, key_secret),
		data={
			"grant_type": "client_credentials",
			"scope": "zela-builder:read zela-builder:write zela-executor:call",
		},
		verify=verify_ssl,
	)
	token_resp.raise_for_status()
	token = token_resp.json().get("access_token")

	if not token:
		raise ValueError("Failed to obtain access token")

	return token


def execute_procedure(
	token: str,
	procedure: str,
	params: dict | list,
	executor_url: str = "https://executor.zela.io",
	route_by: str = "auto",
	verify_ssl: bool = True,
	timeout: float = 30.0,
) -> tuple:
	"""Execute a Zela procedure and return response text and timing."""
	request_start = time.time()

	resp = httpx.post(
		executor_url,
		headers={
			"Authorization": f"Bearer {token}",
			"Content-Type": "application/json",
			"zela-route-by": route_by,
			"zela-route-dbg": "true"
		},
		json={
			"jsonrpc": "2.0",
			"id": 1,
			"method": f"zela.{procedure}",
			"params": params,
		},
		verify=verify_ssl,
		timeout=timeout,
	)
	resp.raise_for_status()

	request_end = time.time()
	req_time = request_end - request_start
	dbg_headers = json.loads(resp.headers.get("zela-routed-dbg"))

	return resp.json(), req_time, dbg_headers


_CACHE_FILE = Path(__file__).parent / ".cache" / "wasm_uploads.json"


def _load_upload_cache() -> dict[str, list[str]]:
	if _CACHE_FILE.exists():
		return json.loads(_CACHE_FILE.read_text())
	return {}


def _save_upload_cache(cache: dict) -> None:
	_CACHE_FILE.parent.mkdir(parents=True, exist_ok=True)
	_CACHE_FILE.write_text(json.dumps(cache, indent=2))


def upload_wasm(
	token: str,
	file_path: Path,
	procedure: str,
	project: str,
	core_url: str,
	verify_ssl: bool = True,
):
	file_path = Path(file_path).resolve()

	assert file_path.is_file(), f"WASM file '{file_path}' does not exist"
	assert file_path.suffix == ".wasm", f"File '{file_path}' is not a .wasm file"

	wasm_data = file_path.read_bytes()
	sha1 = hashlib.sha1(wasm_data).hexdigest()

	cache = _load_upload_cache()
	cache_key = f"{project}/{procedure}"
	if sha1 in cache.get(cache_key, []):
		print(
			f"Skipping upload — '{procedure}' in '{project}' already uploaded (sha1={sha1})"
		)
		return None

	print(
		f"Uploading WASM file '{file_path}' for procedure '{procedure}' in project '{project}'"
	)

	resp = httpx.post(
		f"{core_url}/procedures/{procedure}/wasm",
		headers={
			"Authorization": f"Bearer {token}",
			"Content-Type": "application/wasm",
		},
		params={
			"project": project,
			"file_name": file_path.name,
		},
		content=wasm_data,
		verify=verify_ssl,
	)
	if resp.status_code == 422:
		print(
			f"Status code: {resp.status_code} This procedure hash was already uploaded"
		)
	elif 200 < resp.status_code < 300:
		raise ValueError(f"Procedure execution failed: {resp.status_code} {resp.text}")

	cache.setdefault(cache_key, []).append(sha1)
	_save_upload_cache(cache)

	return resp


def main():
	token, executor_url, core_url = get_zela_vars()
	procedure_path = Path("../target/wasm32-wasip2/release/zela_test_echo.wasm")

	upload_response = upload_wasm(
		token=token,
		file_path=procedure_path,
		procedure="echo1",
		# project="stipl-test",
		project="70adc855-d1a1-4f5c-91dd-80c1b7fda93e",
		core_url=core_url,
		verify_ssl=False,
	)
	print(upload_response)

	sha1 = hashlib.sha1(procedure_path.read_bytes()).hexdigest()
	print(f"SHA-1 hash of WASM file: {sha1}")

	response_text, req_time, dbg_headers = execute_procedure(
		token=token,
		procedure=f"echo2#{sha1}",
		params={"message": "Hello from Zela API!"},
		executor_url=executor_url,
		verify_ssl=False,
	)
	print(response_text)
	print(dbg_headers)
	print(f"\nRequest time: {req_time:.6f}s")


if __name__ == "__main__":
	main()
