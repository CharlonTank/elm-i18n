module Main exposing (..)

view t model =
    div []
        [ h1 [] [ text t.appWelcome ]
        , p [] [ text t.appWelcome ]
        , button [] [ text "Click me" ]
        ]