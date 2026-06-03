use zela_std::{CustomProcedure, JsonValue, RpcError};

pub struct ProcedureRuntimeProcedure;

impl CustomProcedure for ProcedureRuntimeProcedure {
    type Params = JsonValue;
    type SuccessData = JsonValue;
    type ErrorData = ();

    async fn run(params: Self::Params) -> Result<Self::SuccessData, RpcError<Self::ErrorData>> {
        Ok(params)
    }
}

zela_std::zela_custom_procedure!(ProcedureRuntimeProcedure);