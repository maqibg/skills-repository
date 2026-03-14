use anyhow::{anyhow, Context, Result};
use std::path::Path;

use crate::{
    domain::types::{AppSettings, ProxySettings},
    repositories::settings as settings_repository,
};

pub(crate) struct HttpClient {
    agent: ureq::Agent,
    user_agent: String,
}

impl HttpClient {
    pub(crate) fn for_db(db_path: &Path) -> Result<Self> {
        let settings = settings_repository::load_settings(db_path)?
            .unwrap_or_else(|| settings_repository::default_settings("en-US".into()));
        Self::for_settings(&settings)
    }

    pub(crate) fn for_settings(settings: &AppSettings) -> Result<Self> {
        validate_proxy_settings(&settings.proxy)?;
        let agent = build_agent(&settings.proxy)?;
        Ok(Self {
            agent,
            user_agent: default_user_agent(),
        })
    }

    pub(crate) fn get(&self, url: &str) -> ureq::Request {
        self.agent.get(url).set("User-Agent", &self.user_agent)
    }
}

pub(crate) fn default_user_agent() -> String {
    format!("skills-manager/{}", env!("CARGO_PKG_VERSION"))
}

pub(crate) fn validate_proxy_settings(proxy: &ProxySettings) -> Result<()> {
    if !proxy.enabled {
        return Ok(());
    }

    let trimmed = proxy.url.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("proxy url is required when proxy is enabled"));
    }

    normalize_proxy_url(trimmed).map(|_| ())
}

fn build_agent(proxy: &ProxySettings) -> Result<ureq::Agent> {
    let mut builder = ureq::AgentBuilder::new();

    if proxy.enabled {
        let normalized = normalize_proxy_url(proxy.url.trim())?;
        let proxy = ureq::Proxy::new(&normalized)
            .with_context(|| format!("invalid proxy url: {}", normalized))?;
        builder = builder.proxy(proxy);
    }

    Ok(builder.build())
}

fn normalize_proxy_url(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("proxy url is empty"));
    }

    if trimmed.contains("://") {
        return Ok(trimmed.to_string());
    }

    Ok(format!("http://{}", trimmed))
}

