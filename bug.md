# Bug Report: elm-i18n insère les traductions au mauvais endroit dans le record

## Configuration
- **Mode**: multi-file mode avec 3 fichiers de traduction
- **Fichier concerné**: `src/I18n/App.elm` (target: `app`)
- **Langues**: en, fr

## Comportement observé

Lorsque j'ajoute une nouvelle traduction avec :
```bash
elm-i18n --target app add selectedTenantsLabel --en "Selected tenants" --fr "Locataires sélectionnés"
```

L'outil insère la nouvelle clé **au milieu d'une fonction lambda existante** au lieu de la placer au niveau du record.

## Exemple concret

### État du record avant l'ajout

```elm
enTranslations =
    { ...
    , unitStatusRented = "Rented"
    , applicationAutoAcceptedInfo = "Application auto-accepted"
    , applicationStatusLabel = \status -> case status of
            Application.InProgress -> "In progress"
            Application.Accepted -> "Accepted"
            Application.Rejected -> "Rejected"
    , tvReceptionModeLabel = \mode -> ...
    }
```

### Résultat après `elm-i18n add`

```elm
enTranslations =
    { ...
    , unitStatusRented = "Rented"
    , applicationAutoAcceptedInfo = "Application auto-accepted"
    , applicationStatusLabel = \status -> case status of
    , selectedTenantsLabel = "Selected tenants"  -- ❌ INSÉRÉ AU MAUVAIS ENDROIT
            Application.InProgress -> "In progress"
            Application.Accepted -> "Accepted"
            Application.Rejected -> "Rejected"
    , tvReceptionModeLabel = \mode -> ...
    }
```

**Résultat**: Erreur de compilation Elm car la syntaxe est invalide.

```
-- PROBLEM IN PATTERN ----------------------------------------- src/I18n/App.elm

I wanted to parse a pattern next, but I got stuck here:

2506|     , selectedTenantsLabel = "Selected tenants"
          ^
I am not sure why I am getting stuck exactly. I just know that I want a pattern
next. Something as simple as maybeHeight or result would work!
```

## Placement attendu

Les nouvelles clés devraient être insérées **après la fonction complète**, pas au milieu :

```elm
enTranslations =
    { ...
    , unitStatusRented = "Rented"
    , applicationAutoAcceptedInfo = "Application auto-accepted"
    , selectedTenantsLabel = "Selected tenants"  -- ✅ BON ENDROIT
    , applicationStatusLabel = \status -> case status of
            Application.InProgress -> "In progress"
            Application.Accepted -> "Accepted"
            Application.Rejected -> "Rejected"
    , tvReceptionModeLabel = \mode -> ...
    }
```

## Hypothèse

L'outil semble détecter la fin d'une entrée de record en cherchant une ligne commençant par `, `, mais ne prend pas en compte le fait que les fonctions lambda multi-lignes peuvent contenir plusieurs clauses `case of` qui s'étendent sur plusieurs lignes. 

Il insère donc la nouvelle clé dès qu'il trouve le prochain `, ` au lieu d'attendre la fin complète de la fonction lambda.

Le parser devrait probablement :
1. Détecter quand une valeur de champ commence par `\` (lambda)
2. Suivre l'indentation pour savoir quand la lambda se termine
3. Ou compter les parenthèses/accolades pour détecter la fin de l'expression

## Workaround actuel

1. Les traductions sont ajoutées mais au mauvais endroit
2. Je dois manuellement déplacer les nouvelles clés au bon endroit dans le fichier
3. Cela affecte à la fois la version EN et FR du record

## Impact

- Casse la compilation à chaque ajout de traduction quand il y a des fonction translations dans le fichier
- Nécessite une intervention manuelle systématique
- Ralentit le workflow de développement

## Reproductibilité

100% reproductible avec des fichiers contenant des function translations (`add-fn`).
