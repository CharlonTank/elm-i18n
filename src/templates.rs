/// Returns the template for a new I18n.elm file
pub fn get_i18n_template(languages: &[String]) -> String {
    let mut template = String::from(
        r#"module I18n exposing (..)

{-| This module handles internationalization (i18n) for the application.
It provides translations for all UI text in supported languages.
-}


-- TYPES


type Language
"#,
    );

    // Add language variants
    if languages.is_empty() {
        template.push_str("    = EN\n    | FR\n");
    } else {
        for (i, lang) in languages.iter().enumerate() {
            if i == 0 {
                template.push_str(&format!("    = {}\n", lang.to_uppercase()));
            } else {
                template.push_str(&format!("    | {}\n", lang.to_uppercase()));
            }
        }
    }

    template.push_str(
        r#"


type alias Translations =
    { appTitle : String
    , appName : String
    , welcome : String
    , loading : String
    }


-- FUNCTIONS


"#,
    );

    // Add translations for each language
    let langs = if languages.is_empty() {
        vec!["en".to_string(), "fr".to_string()]
    } else {
        languages.to_vec()
    };

    for lang in &langs {
        template.push_str(&format!(
            r#"translations{} : Translations
translations{} =
    {{ appTitle = "{}"
    , appName = "My App"
    , welcome = "{}"
    , loading = "{}"
    }}


"#,
            capitalize_first(lang),
            capitalize_first(lang),
            get_default_title(lang),
            get_default_welcome(lang),
            get_default_loading(lang),
        ));
    }

    // Add helper functions
    template.push_str(&format!(
        r#"{{-| Convert Language to String for storage
-}}
languageToString : Language -> String
languageToString lang =
    case lang of
"#
    ));

    for lang in &langs {
        template.push_str(&format!(
            "        {} ->\n            \"{}\"\n\n",
            lang.to_uppercase(),
            lang
        ));
    }

    template.push_str(&format!(
        r#"

{{-| Convert String to Language with fallback to {}
-}}
stringToLanguage : String -> Language
stringToLanguage str =
    case str of
"#,
        langs[0].to_uppercase()
    ));

    for lang in &langs[1..] {
        template.push_str(&format!(
            "        \"{}\" ->\n            {}\n\n",
            lang,
            lang.to_uppercase()
        ));
    }

    template.push_str(&format!(
        "        _ ->\n            {}\n\n\n",
        langs[0].to_uppercase()
    ));

    // Add translations function
    template.push_str(
        r#"{{-| Get translations for a given language
-}}
translations : Language -> Translations
translations lang =
    case lang of
"#,
    );

    for lang in &langs {
        template.push_str(&format!(
            "        {} ->\n            translations{}\n\n",
            lang.to_uppercase(),
            capitalize_first(lang)
        ));
    }

    template
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn get_default_title(lang: &str) -> &'static str {
    match lang {
        "fr" => "Application Elm",
        "es" => "Aplicación Elm",
        "de" => "Elm Anwendung",
        _ => "Elm Application",
    }
}

fn get_default_welcome(lang: &str) -> &'static str {
    match lang {
        "fr" => "Bienvenue!",
        "es" => "¡Bienvenido!",
        "de" => "Willkommen!",
        _ => "Welcome!",
    }
}

fn get_default_loading(lang: &str) -> &'static str {
    match lang {
        "fr" => "Chargement...",
        "es" => "Cargando...",
        "de" => "Laden...",
        _ => "Loading...",
    }
}