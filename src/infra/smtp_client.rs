use anyhow::Result;
use async_trait::async_trait;
use lettre::AsyncTransport;
use lettre::Tokio1Executor;
use lettre::message::{Mailbox, MessageBuilder};
use lettre::transport::smtp::AsyncSmtpTransport;

pub struct SmtpService {
    from: String,
    smtp_host: String,
    smtp_port: u16,
    smtp_username: String,
    smtp_password: String,
}

impl SmtpService {
    pub fn new(
        from: String,
        smtp_host: String,
        smtp_port: u16,
        smtp_username: String,
        smtp_password: String,
    ) -> Self {
        Self {
            from,
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
        }
    }

    fn build_transport(&self) -> AsyncSmtpTransport<Tokio1Executor> {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&self.smtp_host)
            .unwrap()
            .port(self.smtp_port)
            .credentials(lettre::transport::smtp::authentication::Credentials::new(
                self.smtp_username.clone(),
                self.smtp_password.clone(),
            ))
            .build()
    }
}

#[async_trait]
pub trait SmtpClient: Send + Sync {
    async fn send(&self, to: &str, subject: &str, body: &str) -> Result<()>;
}

#[async_trait]
impl SmtpClient for SmtpService {
    async fn send(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        let from_mailbox = Mailbox::new(None, self.from.parse()?);
        let to_mailbox = Mailbox::new(None, to.parse()?);

        let email = MessageBuilder::new()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(subject)
            .body(String::from(body))?;

        let transport = self.build_transport();
        transport.send(email).await?;

        Ok(())
    }
}
