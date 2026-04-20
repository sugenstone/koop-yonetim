// SMTP mail gönderimi (lettre)
// .env'den okunur: SMTP_HOST, SMTP_PORT, SMTP_USER, SMTP_PASS, SMTP_FROM, ADMIN_EMAIL

use lettre::{
    message::{header::ContentType, Message},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
use std::env;

pub struct MailConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub from: String,
    pub admin_email: String,
}

impl MailConfig {
    pub fn from_env() -> Option<Self> {
        let host = env::var("SMTP_HOST").ok()?;
        let user = env::var("SMTP_USER").ok()?;
        let pass = env::var("SMTP_PASS").ok()?;
        if host.is_empty() || user.is_empty() || pass.is_empty() {
            return None;
        }
        Some(Self {
            host,
            port: env::var("SMTP_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(587),
            user: user.clone(),
            pass,
            from: env::var("SMTP_FROM").unwrap_or(user),
            admin_email: env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@koop.local".into()),
        })
    }
}

/// Mail gönder (non-blocking, hata sadece log'lanır — iş akışını kırmaz).
pub async fn send(to: &str, subject: &str, body_html: &str) {
    let Some(cfg) = MailConfig::from_env() else {
        tracing::warn!("SMTP yapilandirilmamis (SMTP_HOST/USER/PASS eksik) - mail atlandi: {}", subject);
        return;
    };

    let msg = match Message::builder()
        .from(match cfg.from.parse() { Ok(v) => v, Err(e) => { tracing::error!("SMTP_FROM gecersiz: {}", e); return; } })
        .to(match to.parse() { Ok(v) => v, Err(e) => { tracing::error!("Alici mail gecersiz ({}): {}", to, e); return; } })
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(body_html.to_string())
    {
        Ok(m) => m,
        Err(e) => { tracing::error!("Mail olusturulamadi: {}", e); return; }
    };

    let creds = Credentials::new(cfg.user.clone(), cfg.pass.clone());
    let mailer = match AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.host) {
        Ok(b) => b.credentials(creds).port(cfg.port).build(),
        Err(e) => { tracing::error!("SMTP relay olusturulamadi: {}", e); return; }
    };

    match mailer.send(msg).await {
        Ok(_) => tracing::info!("Mail gonderildi -> {}: {}", to, subject),
        Err(e) => tracing::error!("Mail gonderim hatasi ({}): {}", to, e),
    }
}

pub fn admin_email() -> String {
    env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@koop.local".into())
}
