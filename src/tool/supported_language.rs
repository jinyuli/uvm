
pub static GO: &SupportedLanguage = &SupportedLanguage{ name: "go" };
pub static NODE: &SupportedLanguage = &SupportedLanguage{ name: "node" };
pub static JAVA: &SupportedLanguage = &SupportedLanguage{ name: "java" };

#[derive(Debug, Clone)]
pub struct SupportedLanguage {
    pub name: &'static str,
}