import hashlib
from pathlib import Path


from zela_api import execute_procedure, get_zela_vars, upload_wasm


def warm_up_executors(
	token: str,
	executor_url: str,
	procedure_name: str,
	procedure_hash: str,
	params: list | dict,
):
	routes = [
		"static fr2",
		"static tyo",
		"static dx1",
		"static ewr",
		"static slc",
	]
	print("Warming up routes")
	for route in routes:
		response_json, req_time, dbg_headers_json = execute_procedure(
			token=token,
			procedure=f"{procedure_name}#{procedure_hash}",
			params=params,
			executor_url=executor_url,
			verify_ssl=False,
			route_by=route,
		)
		executor = dbg_headers_json.get("executor", {}).get("name")
		router = dbg_headers_json.get("router", {}).get("name")
		print(
			f"[{route}] {executor=}, {router=}\n"
			f"\tclient={req_time:.3f}s\n"
			f"\tserver_rtt={dbg_headers_json.get('rtt')}\n"
			f"\tslot={dbg_headers_json.get('slot')}"
			f"\tresponse={response_json}"
		)


def main():
	# cargo build --target wasm32-wasip2 --release -p miami_procedure; uv run -m warmup
	token, executor_url, core_url = get_zela_vars()
	procedure_path = Path("../target/wasm32-wasip2/release/miami_procedure.wasm")
	procedure_name = "demo-prop-amm"
	upload_response = upload_wasm(
		token=token,
		file_path=procedure_path,
		procedure=procedure_name,
		project="606da726-34ca-4312-af0b-57150e07a334",
		core_url=core_url,
		verify_ssl=False,
	)
	print(upload_response)

	warm_up_executors(
		token=token,
		executor_url=executor_url,
		procedure_name=procedure_name,
		procedure_hash=hashlib.sha1(procedure_path.read_bytes()).hexdigest(),
		params={"pair": "SOL/USDC", "cex_price": 100},
	)


if __name__ == "__main__":
	main()
