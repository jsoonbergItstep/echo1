use std::time::Instant;
use zela_std::{CustomProcedure, JsonValue, RpcError};

pub struct ProcedureRuntimeProcedure;

impl CustomProcedure for ProcedureRuntimeProcedure {
    type Params = JsonValue;
    type SuccessData = JsonValue;
    type ErrorData = ();

    async fn run(_params: Self::Params) -> Result<Self::SuccessData, RpcError<Self::ErrorData>> {
        let start = Instant::now();

        // intentionally minimal work
        let duration_us = start.elapsed().as_micros();

        Ok(serde_json::json!({
            "ok": true,
            "procedure_duration_us": duration_us
        }))
    }
}

zela_std::zela_custom_procedure!(ProcedureRuntimeProcedure);