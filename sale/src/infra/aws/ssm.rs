use crate::errors::Kind::Internal;
use crate::AppResult;

pub struct Adapter {
    cli: aws_sdk_ssm::Client,
    parameter_name: String,
}

impl Adapter {
    pub fn new(cli: aws_sdk_ssm::Client, parameter_name: String) -> Self {
        Adapter {
            cli,
            parameter_name,
        }
    }

    pub async fn load_dotenv(&self) -> AppResult<()> {
        let body = self
            .cli
            .get_parameter()
            .name(self.parameter_name.clone())
            .with_decryption(true)
            .send()
            .await
            .map_err(Internal.from_srcf())?
            .parameter
            .unwrap()
            .value
            .unwrap();

        for (k, v) in
            dotenv_parser::parse_dotenv(&body).map_err(|v| Internal.with(v.to_string()))?
        {
            std::env::set_var(k, v);
        }

        Ok(())
    }
}
