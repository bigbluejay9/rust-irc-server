use chrono;
use futures_cpupool::CpuPool;
use handlebars;
use std;
use super::super::{configuration, templates};

// State that is initialize on server start, but not preconfigured.
// Shared, lock-free, across the binary.
#[derive(Debug)]
pub struct SharedState {
    pub created: chrono::DateTime<chrono::Utc>,
    pub hostname: String,
    pub template_engine: TE,
    pub configuration: std::sync::Arc<configuration::Configuration>,
    pub thread_pool: CpuPool,
}

// Workaround since Handlebars doensn't derive Debug.
pub struct TE(pub handlebars::Handlebars);

impl std::fmt::Debug for TE {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

impl SharedState {
    pub fn new(
        time: chrono::DateTime<chrono::Utc>,
        hostname: String,
        thread_pool: &CpuPool,
        configuration: std::sync::Arc<configuration::Configuration>,
    ) -> Self {
        let mut template_engine = handlebars::Handlebars::new();
        macro_rules! register_template {
            ($name:ident, $template:ident) => {
                template_engine.register_template_string(templates::$name, templates::$template).unwrap();
            }
        }
        // Register all known templates.
        register_template!(DEBUG_TEMPLATE_NAME, DEBUG_HTML_TEMPLATE);
        register_template!(RPL_WELCOME_TEMPLATE_NAME, RPL_WELCOME_TEMPLATE);
        register_template!(RPL_YOURHOST_TEMPLATE_NAME, RPL_YOURHOST_TEMPLATE);
        register_template!(RPL_CREATED_TEMPLATE_NAME, RPL_CREATED_TEMPLATE);

        /* For rapid template iteration (only bin restart required).
        template_engine.register_template_file(
                templates::DEBUG_TEMPLATE_NAME,
                "./template",
            ).unwrap();
       */

        Self {
            created: time,
            hostname: hostname,
            template_engine: TE(template_engine),
            thread_pool: thread_pool.clone(),
            configuration: configuration,
        }
    }
}
