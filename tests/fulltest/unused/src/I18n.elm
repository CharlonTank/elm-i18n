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
    , used : String
    }


-- FUNCTIONS


translationsEn : Translations
translationsEn =
    { appTitle = "Elm Application"
    , used = "Used"
    }


translationsFr : Translations
translationsFr =
    { appTitle = "Application Elm"
    , used = "Utilisé"
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