module I18n exposing (..)

{-| This module handles internationalization (i18n) for the application.
It provides translations for all UI text in supported languages.
-}


-- TYPES


type Language
    = EN
    | FR



type alias LandingTranslations =
    { appTitle : String
    , appName : String
    , welcome : String
    , loading : String
    , heroTitle : String
    }


-- FUNCTIONS


translationsEn : LandingTranslations
translationsEn =
    { appTitle = "Elm Application"
    , appName = "My App"
    , welcome = "Welcome!"
    , heroTitle = "Welcome!"
    , loading = "Loading..."
    }


translationsFr : LandingTranslations
translationsFr =
    { appTitle = "Application Elm"
    , appName = "My App"
    , heroTitle = "Bienvenue!"
    , welcome = "Bienvenue!"
    , loading = "Chargement..."
    }


{-| Convert Language to String for storage
-}
languageToString : Language -> String
languageToString lang =
    case lang of
        EN ->
            "en"

        FR ->
            "fr"



{-| Convert String to Language with fallback to EN
-}
stringToLanguage : String -> Language
stringToLanguage str =
    case str of
        "fr" ->
            FR

        _ ->
            EN


{-| Get translations for a given language
-}
translations : Language -> LandingTranslations
translations lang =
    case lang of
        EN ->
            translationsEn

        FR ->
            translationsFr
