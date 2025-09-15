module I18n exposing (..)

{-| This module handles internationalization (i18n) for the application.
It provides translations for all UI text in supported languages.
-}


-- TYPES


type Language
    = EN
    | FR



type alias AppTranslations =
    { appTitle : String
    , appName : String
    , welcome : String
    , loading : String
    , userProfile : String
    , formatDate : Date -> String
    }


-- FUNCTIONS


translationsEn : AppTranslations
translationsEn =
    { appTitle = "Elm Application"
    , appName = "My App"
    , welcome = "Welcome!"
    , userProfile = "User Profile"
    , formatDate = \\d -> "Date: " ++ dateToString d
    , loading = "Loading..."
    }


translationsFr : AppTranslations
translationsFr =
    { appTitle = "Application Elm"
    , appName = "My App"
    , userProfile = "Profil Utilisateur"
    , formatDate = \\d -> "Date: " ++ dateToString d
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
translations : Language -> AppTranslations
translations lang =
    case lang of
        EN ->
            translationsEn

        FR ->
            translationsFr