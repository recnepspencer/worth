pub use bank_http_adapter::AuthentikAuthorizationCallback;
pub use tokio::io::{AsyncReadExt, AsyncWriteExt};
pub use tokio::net::TcpListener;
pub use url::Url;

pub struct CallbackReceiver {
    pub listener: TcpListener,
    pub redirect_url: String,
}

impl CallbackReceiver {
    pub async fn bind() -> std::io::Result<Self> {
        let listener = TcpListener::bind(("0.0.0.0", 0)).await?;
        let port = listener.local_addr()?.port();
        Ok(Self {
            listener,
            redirect_url: format!("http://host.docker.internal:{port}/callback"),
        })
    }

    pub fn redirect_url(&self) -> String {
        self.redirect_url.clone()
    }

    pub async fn receive(&self) -> Result<ReceivedAuthorizationCallback, String> {
        self.receive_with_response(true).await
    }

    pub async fn receive_without_response(&self) -> Result<ReceivedAuthorizationCallback, String> {
        self.receive_with_response(false).await
    }

    pub async fn receive_with_response(
        &self,
        deliver_response: bool,
    ) -> Result<ReceivedAuthorizationCallback, String> {
        let (mut stream, _) = self
            .listener
            .accept()
            .await
            .map_err(|error| format!("callback accept failed: {error}"))?;
        let mut buffer = [0_u8; 8_192];
        let read = stream
            .read(&mut buffer)
            .await
            .map_err(|error| format!("callback read failed: {error}"))?;
        let request = std::str::from_utf8(&buffer[..read])
            .map_err(|_| "callback request was not UTF-8".to_string())?;
        let target = request
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .ok_or_else(|| "callback request target was missing".to_string())?;
        let callback_url = Url::parse(&format!("http://callback.invalid{target}"))
            .map_err(|error| format!("callback URL was invalid: {error}"))?;
        let code = query_value(&callback_url, "code")?;
        let state = query_value(&callback_url, "state")?;
        if deliver_response {
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 33\r\nConnection: close\r\n\r\nAuthentication callback received.",
                )
                .await
                .map_err(|error| format!("callback response failed: {error}"))?;
        }
        Ok(ReceivedAuthorizationCallback { code, state })
    }
}

pub struct ReceivedAuthorizationCallback {
    pub code: String,
    pub state: String,
}

impl ReceivedAuthorizationCallback {
    pub fn into_authentik(self) -> Result<AuthentikAuthorizationCallback, String> {
        AuthentikAuthorizationCallback::new(self.code, self.state)
            .map_err(|error| format!("callback was rejected: {error}"))
    }

    pub fn with_state(
        self,
        state: impl Into<String>,
    ) -> Result<AuthentikAuthorizationCallback, String> {
        AuthentikAuthorizationCallback::new(self.code, state)
            .map_err(|error| format!("hostile callback was rejected structurally: {error}"))
    }
}

pub fn query_value(url: &Url, name: &str) -> Result<String, String> {
    url.query_pairs()
        .find_map(|(key, value)| (key == name).then(|| value.into_owned()))
        .ok_or_else(|| format!("callback query parameter `{name}` was missing"))
}
