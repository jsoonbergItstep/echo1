use zela_std::{CustomProcedure, JsonValue, RpcError};

pub struct ProcedureRuntimeProcedure;

impl CustomProcedure for ProcedureRuntimeProcedure {
    type Params = JsonValue;
    type SuccessData = JsonValue;
    type ErrorData = ();

    async fn run(_params: Self::Params) -> Result<Self::SuccessData, RpcError<Self::ErrorData>> {
        Ok(serde_json::json!({
            "ok": true,
            "procedure_duration_us": 0
        }))
    }
}

zela_std::zela_custom_procedure!(ProcedureRuntimeProcedure);