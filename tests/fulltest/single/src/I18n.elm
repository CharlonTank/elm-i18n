module I18n exposing (..)

{-| This module handles internationalization (i18n) for the application.
It provides translations for all UI text in supported languages.
-}


-- TYPES


type Language
    = EN
    | FR



type alias Translations =
    { appTitle : String
    , appName : String
    , welcome : String
    , loading : String
    , greeting : String
    , itemCount : Int -> String
    }


-- FUNCTIONS


translationsEn : Translations
translationsEn =
    { appTitle = "Elm Application"
    , appName = "My App"
    , welcome = "Welcome!"
    , greeting = "Hello"
    , itemCount = \\n -> String.fromInt n ++ " items"
    , loading = "Loading..."
    }


translationsFr : Translations
translationsFr =
    { appTitle = "Application Elm"
    , appName = "My App"
    , greeting = "Bonjour"
    , itemCount = \\n -> String.fromInt n ++ " articles"
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
translations : Language -> Translations
translations lang =
    case lang of
        EN ->
            translationsEn

        FR ->
            translationsFr