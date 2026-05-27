import hashlib
import os
from pathlib import Path


from zela_api import execute_procedure, get_zela_vars, upload_wasm


def test_wrong_pair(token, executor_url, procedure_path):
	sha1 = hashlib.sha1(procedure_path.read_bytes()).hexdigest()
	response_text, req_time = execute_procedure(
		token=token,
		procedure=f"miami_demo#{sha1}",
		params={"pair": "SOL/USDT", "cex_price": 0},
		executor_url=executor_url,
		verify_ssl=False,
	)

	error = response_text.get("error")
	assert error
	assert "Only supported pair is 'SOL/USDC'" in error.get("message"), error


def test_wrong_price(token, executor_url, procedure_path):
	sha1 = hashlib.sha1(procedure_path.read_bytes()).hexdigest()
	response_text, req_time = execute_procedure(
		token=token,
		procedure=f"miami_demo#{sha1}",
		params={"pair": "SOL/USDC", "cex_price": -1},
		executor_url=executor_url,
		verify_ssl=False,
	)

	error = response_text.get("error")
	assert error
	assert "CEX price can not be negative" in error.get("message")


def main():
	token, executor_url, core_url = get_zela_vars()
	procedure_path = Path("../target/wasm32-wasip2/release/miami_procedure.wasm")

	upload_response = upload_wasm(
		token=token,
		file_path=procedure_path,
		procedure="miami_demo",
		project="70adc855-d1a1-4f5c-91dd-80c1b7fda93e",
		core_url=core_url,
		verify_ssl=False,
	)
	print(upload_response)

	sha1 = hashlib.sha1(procedure_path.read_bytes()).hexdigest()
	print(f"SHA-1 hash of WASM file: {sha1}")

	response_text, req_time = execute_procedure(
		token=token,
		procedure=f"miami_demo#{sha1}",
		params={"pair": "SOL/USDC", "cex_price": 100},
		executor_url=executor_url,
		verify_ssl=False,
	)
	print(response_text)
	print(f"\nRequest time: {req_time:.6f}s")

	test_wrong_pair(token, executor_url, procedure_path)
	test_wrong_price(token, executor_url, procedure_path)


if __name__ == "__main__":
	main()
