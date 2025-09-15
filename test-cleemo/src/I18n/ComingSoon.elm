module I18n.ComingSoon exposing (ComingSoonTranslations, en, fr)

-- elm-i18n translation file: This module handles internationalization (i18n) for the coming soon page.
-- IMPORTANT: Do not modify the translations directly, instead use the `elm-i18n` cli tool to update the translations.


type alias ComingSoonTranslations =
    { -- Landing Page
      cleemo : String
    , futureOfPropertyManagement : String
    , platformUnderDevelopment : String
    , availableEndOf2025 : String
    , cleemoComingSoon : String
    , tagline : String
    , shapeTheFuture : String
    , weAreDeveloping : String
    , quickSignup : String
    , quickSignupDesc : String
    , emailPlaceholder : String
    , subscribe : String
    , detailedQuestionnaire : String
    , questionnaireTime : String
    , participateInQuestionnaire : String
    , saveTime : String
    , saveTimeDesc : String
    , centralize : String
    , centralizeDesc : String
    , simplify : String
    , simplifyDesc : String

    -- Success View
    , thankYou : String
    , successMessage : String
    , followUpMessage : String

    -- Roles Page
    , helpUsCreateCleemo : String
    , propertyManagementSaves : String
    , tellUsYourReality : String
    , yourRolesToday : String
    , roleOwner : String
    , roleTenant : String
    , roleAgency : String
    , roleVendor : String
    , roleCandidate : String
    , other : String
    , specify : String
    , wantToTestCleemo : String
    , yesAsap : String
    , maybeLearnMore : String
    , notNow : String
    , howToContact : String
    , phonePlaceholder : String
    , acceptDemo : String
    , acceptBeta : String
    , acceptInterview : String

    -- Owner Module
    , ownerModuleStep : Int -> Int -> String
    , profileOrganization : String
    , howManyProperties : String
    , howManyActiveLeases : String
    , rentalTypes : String
    , rentalUnfurnished : String
    , rentalFurnished : String
    , rentalShared : String
    , rentalSeasonal : String
    , rentalMobility : String
    , rentalCommercial : String
    , toolsAndTime : String
    , currentTools : String
    , weeklyTimeSpent : String
    , communicationTickets : String
    , phoneDiscomfort : String
    , notAtAll : String
    , enormously : String
    , singleChannelWilling : String
    , yesWithoutHesitation : String
    , yesIfSimple : String
    , notSure : String
    , no : String
    , exchangeVolume : String
    , messages : String
    , calls : String
    , featurePrioritization : String
    , topTicketCategories : String
    , max3Categories : String
    , heating : String
    , electricity : String
    , plumbing : String
    , keysAccess : String
    , pests : String
    , insurance : String
    , noise : String
    , rentReceipts : String
    , leaseDocuments : String
    , evaluateFeatures : String
    , adoptionPricing : String
    , importTenants : String
    , allFromStart : String
    , progressively : String
    , onlyNewOnes : String
    , dontKnowYet : String
    , bankConnection : String
    , comfortableWithIt : String
    , maybeLater : String
    , noThanks : String
    , autoVendorContact : String
    , yesFromMyList : String
    , yesFromMarketplace : String
    , noKeepControl : String
    , biggestIrritant : String
    , irritantPlaceholder : String
    , pricingPerception : String
    , tooCheap : String
    , goodDeal : String
    , gettingExpensive : String
    , tooExpensive : String
    , pricePlaceholder : String
    , priceUnit : String
    , previous : String
    , back : String
    , next : String
    , finishModule : String

    -- Tenant Module
    , tenantModule : String
    , helpUsUnderstandTenant : String
    , acceptUniqueApp : String
    , yesNoProblem : String
    , yesIfEasy : String
    , idontKnow : String
    , noPreferMyChannels : String
    , yourPriorities : String
    , max3Priorities : String
    , quickResponse : String
    , clearFollowup : String
    , messageHistory : String
    , easyPhotos : String
    , documentAccess : String
    , fewerCalls : String
    , guidedSupport : String
    , yesHelpful : String
    , maybeDependsOnProblem : String
    , noPreferPerson : String
    , preferredChannel : String
    , dedicatedApp : String
    , email : String
    , smsWhatsapp : String
    , doesntMatter : String

    -- Agency Module
    , agencyModule : String
    , aboutYourAgency : String
    , companySize : String
    , solo : String
    , people2to10 : String
    , people11to50 : String
    , people51to200 : String
    , people200plus : String
    , currentSoftware : String
    , softwarePlaceholder : String
    , agencyModeInterest : String
    , veryInterested : String
    , moderatelyInterested : String
    , littleInterested : String
    , professionalContact : String
    , agencyEmailPlaceholder : String

    -- Vendor Module
    , vendorModule : String
    , joinOurNetwork : String
    , yourTrade : String
    , tradePlaceholder : String
    , serviceArea : String
    , areaPlaceholder : String
    , acceptJobsViaCleemo : String
    , yesInterested : String
    , notForNow : String
    , vendorEmailPlaceholder : String

    -- Candidate Module
    , candidateModule : String
    , simplifyYourSearch : String
    , reusableProfileInterest : String
    , createProfileOnce : String
    , veryInterestedCandidate : String
    , unsureCandidate : String
    , notInterestedCandidate : String
    , mainConcerns : String
    , concernsPlaceholder : String

    -- Closing Page
    , oneLastThing : String
    , yourFeedbackHelps : String
    , whatShouldWeHaveAsked : String
    , anythingImportantForgotten : String
    , yourOpinionPlaceholder : String
    , howToParticipate : String
    , dataProtected : String
    , neverShareWithoutConsent : String
    , finishAndSend : String

    -- Checkboxes
    , testBeta : String
    , participate30minInterview : String
    , joinEarlyAdopters : String

    -- Common Trades
    , plumber : String
    , electrician : String
    , locksmith : String
    , multiServices : String

    -- Rating Labels
    , importanceRating : String
    , solvesMyProblem : String

    -- Time ranges
    , timeLess1h : String
    , time1to3h : String
    , time4to6h : String
    , time7to10h : String
    , timeMore10h : String

    -- Property/lease ranges
    , range0 : String
    , range1 : String
    , range2to4 : String
    , range5to9 : String
    , range10to19 : String
    , range20plus : String

    -- Volume ranges (messages)
    , vol0to2 : String
    , vol3to5 : String
    , vol6to10 : String
    , vol11to20 : String
    , vol20plus : String

    -- Volume ranges (calls)
    , call0to1 : String
    , call2to3 : String
    , call4to6 : String
    , call7to10 : String
    , call10plus : String

    -- Features (Owner module)
    , feature1Title : String
    , feature1Desc : String
    , feature2Title : String
    , feature2Desc : String
    , feature3Title : String
    , feature3Desc : String
    , feature4Title : String
    , feature4Desc : String
    , feature5Title : String
    , feature5Desc : String
    , feature6Title : String
    , feature6Desc : String
    , feature7Title : String
    , feature7Desc : String
    , feature8Title : String
    , feature8Desc : String
    , feature9Title : String
    , feature9Desc : String
    }


fr : ComingSoonTranslations
fr =
    { -- Landing Page
      cleemo = "Cleemo"
    , futureOfPropertyManagement = "Le futur de la gestion immobilière"
    , platformUnderDevelopment = "Notre plateforme est en développement"
    , availableEndOf2025 = "Disponible fin 2025"
    , cleemoComingSoon = "Cleemo arrive bientôt"
    , tagline = "La gestion locative simple et efficace"
    , shapeTheFuture = "Façonnez l'avenir de la gestion locative"
    , weAreDeveloping = "Nous développons Cleemo pour vous simplifier la vie. Vos retours sont essentiels pour créer l'outil qui répond vraiment à vos besoins quotidiens."
    , quickSignup = "📧 Inscription rapide"
    , quickSignupDesc = "Laissez votre email pour être informé·e du lancement"
    , emailPlaceholder = "votre@email.com"
    , subscribe = "S'inscrire"
    , detailedQuestionnaire = "🎯 Questionnaire détaillé"
    , questionnaireTime = "5-8 minutes pour influencer nos priorités de développement"
    , participateInQuestionnaire = "Participer au questionnaire"
    , saveTime = "Gagnez du temps"
    , saveTimeDesc = "Automatisez les tâches répétitives et concentrez-vous sur l'essentiel"
    , centralize = "Centralisez"
    , centralizeDesc = "Un seul outil pour gérer tous vos échanges avec les locataires"
    , simplify = "Simplifiez"
    , simplifyDesc = "Dashboard intuitif pour suivre vos biens et vos revenus"

    -- Success View
    , thankYou = "Merci !"
    , successMessage = "Vous venez d'influencer directement les priorités du produit."
    , followUpMessage = "On vous tient au courant si vous l'avez demandé — sinon, pas d'email, pas de spam."

    -- Roles Page
    , helpUsCreateCleemo = "Aidez-nous à créer Cleemo"
    , propertyManagementSaves = "La gestion locative qui vous fait gagner du temps"
    , tellUsYourReality = "Parlez-nous de votre réalité. En quelques minutes, vous nous aidez à cibler l'essentiel."
    , yourRolesToday = "Vos rôles aujourd'hui (cochez tout ce qui s'applique)"
    , roleOwner = "Propriétaire bailleur (je loue au moins 1 bien)"
    , roleTenant = "Locataire"
    , roleAgency = "Agence (micro/PME/grande)"
    , roleVendor = "Prestataire (plombier, électricien, multiservice...)"
    , roleCandidate = "Candidat (je cherche un logement)"
    , other = "Autre: "
    , specify = "Précisez..."
    , wantToTestCleemo = "Envie de tester Cleemo en avant-première ?"
    , yesAsap = "Oui, dès que possible"
    , maybeLearnMore = "Peut-être, j'aimerais d'abord en savoir plus"
    , notNow = "Pas pour l'instant"
    , howToContact = "Comment préférez-vous être contacté·e ? (facultatif)"
    , phonePlaceholder = "06 12 34 56 78"
    , acceptDemo = "J'accepte d'être recontacté·e pour une démo"
    , acceptBeta = "J'accepte d'être invité·e à la bêta"
    , acceptInterview = "J'accepte d'être sollicité·e pour une courte interview utilisateur"

    -- Owner Module
    , ownerModuleStep = \current total -> "Module Propriétaire - Étape " ++ String.fromInt current ++ "/" ++ String.fromInt total
    , profileOrganization = "Profil & Organisation"
    , howManyProperties = "Combien de biens loués gérez-vous vous-même (hors agence) ?"
    , howManyActiveLeases = "En tout, combien de lots (baux) actifs ?"
    , rentalTypes = "Types de location (plusieurs choix possibles)"
    , rentalUnfurnished = "Nue"
    , rentalFurnished = "Meublée"
    , rentalShared = "Colocation"
    , rentalSeasonal = "Saisonnière/LD"
    , rentalMobility = "Bail mobilité"
    , rentalCommercial = "Commerciale"
    , toolsAndTime = "Outils & Temps"
    , currentTools = "Outils actuels (plusieurs choix possibles)"
    , weeklyTimeSpent = "Temps passé par semaine sur la gestion locative (estimation)"
    , communicationTickets = "Communication & Tickets"
    , phoneDiscomfort = "Partager votre numéro perso avec les locataires vous gêne-t-il ?"
    , notAtAll = "0 - Pas du tout"
    , enormously = "10 - Énormément"
    , singleChannelWilling = "Seriez-vous prêt·e à imposer un canal unique (Cleemo) pour tous les échanges ?"
    , yesWithoutHesitation = "Oui, sans hésiter"
    , yesIfSimple = "Oui si c'est simple pour eux"
    , notSure = "Pas sûr"
    , no = "Non"
    , exchangeVolume = "Volume actuel d'échanges par lot et par mois (estimation)"
    , messages = "Messages (écrit)"
    , calls = "Appels"
    , featurePrioritization = "Priorisation des fonctionnalités"
    , topTicketCategories = "Top catégories de tickets qui vous font perdre du temps (3 max)"
    , max3Categories = "Maximum 3 catégories, merci de déselectionner."
    , heating = "Chauffage/Clim"
    , electricity = "Électricité"
    , plumbing = "Plomberie/Eau"
    , keysAccess = "Clés/Accès"
    , pests = "Nuisibles"
    , insurance = "Assurance/Sinistres"
    , noise = "Bruits/Voisinage"
    , rentReceipts = "Demandes quittances"
    , leaseDocuments = "Docs bail/CAF"
    , evaluateFeatures = "Évaluez ces fonctionnalités"
    , adoptionPricing = "Adoption & Tarification"
    , importTenants = "Import de vos locataires existants dans Cleemo"
    , allFromStart = "Tous dès le début"
    , progressively = "Progressivement"
    , onlyNewOnes = "Seulement les nouveaux"
    , dontKnowYet = "Je ne sais pas encore"
    , bankConnection = "Connexion bancaire pour automatiser (quittances, rappels)"
    , comfortableWithIt = "Confortable avec ça"
    , maybeLater = "Peut-être plus tard"
    , noThanks = "Non merci"
    , autoVendorContact = "Contact automatique d'un prestataire si urgence détectée"
    , yesFromMyList = "Oui, depuis ma liste de prestataires"
    , yesFromMarketplace = "Oui, depuis la marketplace Cleemo"
    , noKeepControl = "Non, je veux garder le contrôle"
    , biggestIrritant = "Votre plus gros irritant actuel (optionnel)"
    , irritantPlaceholder = "Ex: Les relances pour les quittances, les appels le weekend..."
    , pricingPerception = "Perception du prix (par mois, par lot géré)"
    , tooCheap = "À quel prix ce serait trop bon marché (doute sur la qualité) ?"
    , goodDeal = "À quel prix ce serait une bonne affaire ?"
    , gettingExpensive = "À quel prix ça commence à être cher (mais acceptable) ?"
    , tooExpensive = "À quel prix c'est trop cher (vous n'achèteriez pas) ?"
    , pricePlaceholder = "Ex: 5.00"
    , priceUnit = "€ / mois / lot"
    , previous = "Précédent"
    , back = "Retour"
    , next = "Suivant"
    , finishModule = "Terminer module"

    -- Tenant Module
    , tenantModule = "Module Locataire"
    , helpUsUnderstandTenant = "Aidez-nous à comprendre vos besoins en tant que locataire"
    , acceptUniqueApp = "Accepteriez-vous une app unique imposée par votre propriétaire ?"
    , yesNoProblem = "Oui, sans problème"
    , yesIfEasy = "Oui si c'est simple à utiliser"
    , idontKnow = "Je ne sais pas"
    , noPreferMyChannels = "Non, je préfère mes canaux habituels"
    , yourPriorities = "Vos priorités (3 maximum)"
    , max3Priorities = "Maximum 3 priorités, merci de déselectionner."
    , quickResponse = "Réponse rapide du propriétaire"
    , clearFollowup = "Suivi clair de mes demandes"
    , messageHistory = "Historique des échanges"
    , easyPhotos = "Envoi facile photos/vidéos"
    , documentAccess = "Accès aux documents (bail, quittances)"
    , fewerCalls = "Moins d'appels téléphoniques"
    , guidedSupport = "Seriez-vous OK pour un support guidé (pas-à-pas, vidéos) ?"
    , yesHelpful = "Oui, ça m'aiderait"
    , maybeDependsOnProblem = "Peut-être, selon le problème"
    , noPreferPerson = "Non, je préfère parler à quelqu'un"
    , preferredChannel = "Votre canal de communication préféré"
    , dedicatedApp = "App dédiée"
    , email = "Email"
    , smsWhatsapp = "SMS/WhatsApp"
    , doesntMatter = "Peu importe"

    -- Agency Module
    , agencyModule = "Module Agence"
    , aboutYourAgency = "Quelques questions sur votre agence"
    , companySize = "Taille de votre entreprise"
    , solo = "Solo"
    , people2to10 = "2-10 personnes"
    , people11to50 = "11-50 personnes"
    , people51to200 = "51-200 personnes"
    , people200plus = "200+ personnes"
    , currentSoftware = "Vos logiciels actuels (optionnel)"
    , softwarePlaceholder = "Ex: ImmoFacile, RentManager, Excel..."
    , agencyModeInterest = "Intérêt pour un mode agence dédié"
    , veryInterested = "Très intéressé"
    , moderatelyInterested = "Moyennement intéressé"
    , littleInterested = "Peu intéressé"
    , professionalContact = "Contact professionnel (optionnel)"
    , agencyEmailPlaceholder = "contact@agence.fr"

    -- Vendor Module
    , vendorModule = "Module Prestataire"
    , joinOurNetwork = "Rejoignez notre réseau de prestataires"
    , yourTrade = "Votre métier/spécialité"
    , tradePlaceholder = "Ex: Plombier, Électricien, Peintre..."
    , serviceArea = "Zone d'intervention"
    , areaPlaceholder = "Ex: Paris et petite couronne, Lyon 69000..."
    , acceptJobsViaCleemo = "Accepteriez-vous de recevoir des demandes via Cleemo ?"
    , yesInterested = "Oui, intéressé"
    , notForNow = "Non, pas pour le moment"
    , vendorEmailPlaceholder = "contact@entreprise.fr"

    -- Candidate Module
    , candidateModule = "Module Candidat"
    , simplifyYourSearch = "Simplifiez votre recherche de logement"
    , reusableProfileInterest = "Un profil réutilisable vous intéresse-t-il ?"
    , createProfileOnce = "Créez votre dossier une fois, envoyez-le à plusieurs propriétaires"
    , veryInterestedCandidate = "Très intéressé"
    , unsureCandidate = "Je ne sais pas"
    , notInterestedCandidate = "Pas intéressé"
    , mainConcerns = "Vos principales inquiétudes (max 2)"
    , concernsPlaceholder = "Ex: sécurité des données, adoption par les propriétaires, facilité d'utilisation, coût..."

    -- Closing Page
    , oneLastThing = "Une dernière chose..."
    , yourFeedbackHelps = "Vos retours nous aident à construire le produit qui vous correspond"
    , whatShouldWeHaveAsked = "Qu'aurions-nous dû vous demander ?"
    , anythingImportantForgotten = "Y a-t-il quelque chose d'important que nous avons oublié ?"
    , yourOpinionPlaceholder = "Votre avis nous intéresse..."
    , howToParticipate = "Comment souhaitez-vous participer ?"
    , dataProtected = "Vos données sont protégées conformément au RGPD."
    , neverShareWithoutConsent = "Nous ne les partagerons jamais sans votre consentement explicite."
    , finishAndSend = "Terminer et envoyer 🎯"

    -- Checkboxes
    , testBeta = "🚀 Tester la version bêta en avant-première"
    , participate30minInterview = "💬 Participer à un entretien de 30 minutes"
    , joinEarlyAdopters = "👥 Rejoindre la communauté des early adopters"

    -- Common Trades
    , plumber = "Plombier"
    , electrician = "Électricien"
    , locksmith = "Serrurier"
    , multiServices = "Multi-services"

    -- Rating Labels
    , importanceRating = "Importance (1=faible, 5=critique)"
    , solvesMyProblem = "Résout mon problème (1=peu, 5=totalement)"

    -- Time ranges
    , timeLess1h = "<1h"
    , time1to3h = "1-3h"
    , time4to6h = "4-6h"
    , time7to10h = "7-10h"
    , timeMore10h = ">10h"

    -- Property/lease ranges
    , range0 = "0"
    , range1 = "1"
    , range2to4 = "2-4"
    , range5to9 = "5-9"
    , range10to19 = "10-19"
    , range20plus = "20+"

    -- Volume ranges (messages)
    , vol0to2 = "0-2"
    , vol3to5 = "3-5"
    , vol6to10 = "6-10"
    , vol11to20 = "11-20"
    , vol20plus = "20+"

    -- Volume ranges (calls)
    , call0to1 = "0-1"
    , call2to3 = "2-3"
    , call4to6 = "4-6"
    , call7to10 = "7-10"
    , call10plus = "10+"

    -- Features (Owner module)
    , feature1Title = "Chat centralisé avec tous vos locataires"
    , feature1Desc = "Un seul canal pour tous les échanges"
    , feature2Title = "Numéro masqué pour votre tranquillité"
    , feature2Desc = "Protégez votre numéro perso"
    , feature3Title = "Tickets incidents avec suivi en temps réel"
    , feature3Desc = "Plus de messages perdus"
    , feature4Title = "Envoi de quittances automatique"
    , feature4Desc = "Finies les relances"
    , feature5Title = "Rappels de paiement automatisés"
    , feature5Desc = "Respectueux mais efficaces"
    , feature6Title = "Connexion bancaire pour détection paiements"
    , feature6Desc = "Automatisation comptable"
    , feature7Title = "Marketplace de prestataires qualifiés"
    , feature7Desc = "Interventions rapides"
    , feature8Title = "Diagnostics guidés avec IA"
    , feature8Desc = "Résoudre avant d'appeler un pro"
    , feature9Title = "Signature électronique intégrée"
    , feature9Desc = "Baux, avenants, états des lieux"
    }


en : ComingSoonTranslations
en =
    { -- Landing Page
      cleemo = "Cleemo"
    , futureOfPropertyManagement = "The future of property management"
    , platformUnderDevelopment = "Our platform is under development"
    , availableEndOf2025 = "Available end of 2025"
    , cleemoComingSoon = "Cleemo is coming soon"
    , tagline = "Simple and efficient property management"
    , shapeTheFuture = "Shape the future of property management"
    , weAreDeveloping = "We're developing Cleemo to simplify your life. Your feedback is essential to create the tool that truly meets your daily needs."
    , quickSignup = "📧 Quick signup"
    , quickSignupDesc = "Leave your email to be notified at launch"
    , emailPlaceholder = "your@email.com"
    , subscribe = "Subscribe"
    , detailedQuestionnaire = "🎯 Detailed questionnaire"
    , questionnaireTime = "5-8 minutes to influence our development priorities"
    , participateInQuestionnaire = "Participate in questionnaire"
    , saveTime = "Save time"
    , saveTimeDesc = "Automate repetitive tasks and focus on what matters"
    , centralize = "Centralize"
    , centralizeDesc = "One tool to manage all your exchanges with tenants"
    , simplify = "Simplify"
    , simplifyDesc = "Intuitive dashboard to track your properties and income"

    -- Success View
    , thankYou = "Thank you!"
    , successMessage = "You've just directly influenced the product priorities."
    , followUpMessage = "We'll keep you posted if you requested it — otherwise, no email, no spam."

    -- Roles Page
    , helpUsCreateCleemo = "Help us create Cleemo"
    , propertyManagementSaves = "Property management that saves you time"
    , tellUsYourReality = "Tell us about your reality. In a few minutes, you help us target what's essential."
    , yourRolesToday = "Your roles today (check all that apply)"
    , roleOwner = "Property owner (I rent at least 1 property)"
    , roleTenant = "Tenant"
    , roleAgency = "Agency (micro/SME/large)"
    , roleVendor = "Service provider (plumber, electrician, multi-service...)"
    , roleCandidate = "Candidate (I'm looking for housing)"
    , other = "Other: "
    , specify = "Specify..."
    , wantToTestCleemo = "Want to test Cleemo early?"
    , yesAsap = "Yes, as soon as possible"
    , maybeLearnMore = "Maybe, I'd like to learn more first"
    , notNow = "Not right now"
    , howToContact = "How would you prefer to be contacted? (optional)"
    , phonePlaceholder = "+33 6 12 34 56 78"
    , acceptDemo = "I agree to be contacted for a demo"
    , acceptBeta = "I agree to be invited to the beta"
    , acceptInterview = "I agree to be contacted for a short user interview"

    -- Owner Module
    , ownerModuleStep = \current total -> "Owner Module - Step " ++ String.fromInt current ++ "/" ++ String.fromInt total
    , profileOrganization = "Profile & Organization"
    , howManyProperties = "How many rental properties do you manage yourself (excluding agency)?"
    , howManyActiveLeases = "In total, how many active units (leases)?"
    , rentalTypes = "Rental types (multiple choices possible)"
    , rentalUnfurnished = "Unfurnished"
    , rentalFurnished = "Furnished"
    , rentalShared = "Shared housing"
    , rentalSeasonal = "Seasonal/Short-term"
    , rentalMobility = "Mobility lease"
    , rentalCommercial = "Commercial"
    , toolsAndTime = "Tools & Time"
    , currentTools = "Current tools (multiple choices possible)"
    , weeklyTimeSpent = "Time spent per week on property management (estimate)"
    , communicationTickets = "Communication & Tickets"
    , phoneDiscomfort = "Does sharing your personal number with tenants bother you?"
    , notAtAll = "0 - Not at all"
    , enormously = "10 - Enormously"
    , singleChannelWilling = "Would you be willing to impose a single channel (Cleemo) for all exchanges?"
    , yesWithoutHesitation = "Yes, without hesitation"
    , yesIfSimple = "Yes if it's simple for them"
    , notSure = "Not sure"
    , no = "No"
    , exchangeVolume = "Current volume of exchanges per unit per month (estimate)"
    , messages = "Messages (written)"
    , calls = "Calls"
    , featurePrioritization = "Feature Prioritization"
    , topTicketCategories = "Top ticket categories that waste your time (3 max)"
    , max3Categories = "Maximum 3 categories, please deselect."
    , heating = "Heating/AC"
    , electricity = "Electricity"
    , plumbing = "Plumbing/Water"
    , keysAccess = "Keys/Access"
    , pests = "Pests"
    , insurance = "Insurance/Claims"
    , noise = "Noise/Neighbors"
    , rentReceipts = "Rent receipt requests"
    , leaseDocuments = "Lease docs/CAF"
    , evaluateFeatures = "Evaluate these features"
    , adoptionPricing = "Adoption & Pricing"
    , importTenants = "Import your existing tenants into Cleemo"
    , allFromStart = "All from the start"
    , progressively = "Progressively"
    , onlyNewOnes = "Only new ones"
    , dontKnowYet = "I don't know yet"
    , bankConnection = "Bank connection to automate (receipts, reminders)"
    , comfortableWithIt = "Comfortable with it"
    , maybeLater = "Maybe later"
    , noThanks = "No thanks"
    , autoVendorContact = "Automatic vendor contact if emergency detected"
    , yesFromMyList = "Yes, from my vendor list"
    , yesFromMarketplace = "Yes, from Cleemo marketplace"
    , noKeepControl = "No, I want to keep control"
    , biggestIrritant = "Your biggest current irritant (optional)"
    , irritantPlaceholder = "E.g.: Receipt reminders, weekend calls..."
    , pricingPerception = "Price perception (per month, per managed unit)"
    , tooCheap = "At what price would it be too cheap (quality concerns)?"
    , goodDeal = "At what price would it be a good deal?"
    , gettingExpensive = "At what price does it start getting expensive (but acceptable)?"
    , tooExpensive = "At what price is it too expensive (you wouldn't buy)?"
    , pricePlaceholder = "E.g.: 5.00"
    , priceUnit = "€ / month / unit"
    , previous = "Previous"
    , back = "Back"
    , next = "Next"
    , finishModule = "Finish module"

    -- Tenant Module
    , tenantModule = "Tenant Module"
    , helpUsUnderstandTenant = "Help us understand your needs as a tenant"
    , acceptUniqueApp = "Would you accept a unique app imposed by your landlord?"
    , yesNoProblem = "Yes, no problem"
    , yesIfEasy = "Yes if it's easy to use"
    , idontKnow = "I don't know"
    , noPreferMyChannels = "No, I prefer my usual channels"
    , yourPriorities = "Your priorities (3 maximum)"
    , max3Priorities = "Maximum 3 priorities, please deselect."
    , quickResponse = "Quick response from landlord"
    , clearFollowup = "Clear follow-up on my requests"
    , messageHistory = "Message history"
    , easyPhotos = "Easy photo/video sending"
    , documentAccess = "Access to documents (lease, receipts)"
    , fewerCalls = "Fewer phone calls"
    , guidedSupport = "Would you be OK with guided support (step-by-step, videos)?"
    , yesHelpful = "Yes, it would help"
    , maybeDependsOnProblem = "Maybe, depends on the problem"
    , noPreferPerson = "No, I prefer talking to someone"
    , preferredChannel = "Your preferred communication channel"
    , dedicatedApp = "Dedicated app"
    , email = "Email"
    , smsWhatsapp = "SMS/WhatsApp"
    , doesntMatter = "Doesn't matter"

    -- Agency Module
    , agencyModule = "Agency Module"
    , aboutYourAgency = "Some questions about your agency"
    , companySize = "Company size"
    , solo = "Solo"
    , people2to10 = "2-10 people"
    , people11to50 = "11-50 people"
    , people51to200 = "51-200 people"
    , people200plus = "200+ people"
    , currentSoftware = "Your current software (optional)"
    , softwarePlaceholder = "E.g.: Property Manager, RentTracker, Excel..."
    , agencyModeInterest = "Interest in a dedicated agency mode"
    , veryInterested = "Very interested"
    , moderatelyInterested = "Moderately interested"
    , littleInterested = "Little interested"
    , professionalContact = "Professional contact (optional)"
    , agencyEmailPlaceholder = "contact@agency.com"

    -- Vendor Module
    , vendorModule = "Service Provider Module"
    , joinOurNetwork = "Join our network of service providers"
    , yourTrade = "Your trade/specialty"
    , tradePlaceholder = "E.g.: Plumber, Electrician, Painter..."
    , serviceArea = "Service area"
    , areaPlaceholder = "E.g.: Paris and suburbs, Lyon 69000..."
    , acceptJobsViaCleemo = "Would you accept receiving requests via Cleemo?"
    , yesInterested = "Yes, interested"
    , notForNow = "No, not for now"
    , vendorEmailPlaceholder = "contact@company.com"

    -- Candidate Module
    , candidateModule = "Candidate Module"
    , simplifyYourSearch = "Simplify your housing search"
    , reusableProfileInterest = "Would a reusable profile interest you?"
    , createProfileOnce = "Create your file once, send it to multiple landlords"
    , veryInterestedCandidate = "Very interested"
    , unsureCandidate = "I don't know"
    , notInterestedCandidate = "Not interested"
    , mainConcerns = "Your main concerns (max 2)"
    , concernsPlaceholder = "E.g.: data security, landlord adoption, ease of use, cost..."

    -- Closing Page
    , oneLastThing = "One last thing..."
    , yourFeedbackHelps = "Your feedback helps us build the product that fits you"
    , whatShouldWeHaveAsked = "What should we have asked you?"
    , anythingImportantForgotten = "Is there something important we forgot?"
    , yourOpinionPlaceholder = "Your opinion matters..."
    , howToParticipate = "How would you like to participate?"
    , dataProtected = "Your data is protected in accordance with GDPR."
    , neverShareWithoutConsent = "We will never share it without your explicit consent."
    , finishAndSend = "Finish and send 🎯"

    -- Checkboxes
    , testBeta = "🚀 Test the beta version early"
    , participate30minInterview = "💬 Participate in a 30-minute interview"
    , joinEarlyAdopters = "👥 Join the early adopters community"

    -- Common Trades
    , plumber = "Plumber"
    , electrician = "Electrician"
    , locksmith = "Locksmith"
    , multiServices = "Multi-services"

    -- Rating Labels
    , importanceRating = "Importance (1=low, 5=critical)"
    , solvesMyProblem = "Solves my problem (1=little, 5=completely)"

    -- Time ranges
    , timeLess1h = "<1h"
    , time1to3h = "1-3h"
    , time4to6h = "4-6h"
    , time7to10h = "7-10h"
    , timeMore10h = ">10h"

    -- Property/lease ranges
    , range0 = "0"
    , range1 = "1"
    , range2to4 = "2-4"
    , range5to9 = "5-9"
    , range10to19 = "10-19"
    , range20plus = "20+"

    -- Volume ranges (messages)
    , vol0to2 = "0-2"
    , vol3to5 = "3-5"
    , vol6to10 = "6-10"
    , vol11to20 = "11-20"
    , vol20plus = "20+"

    -- Volume ranges (calls)
    , call0to1 = "0-1"
    , call2to3 = "2-3"
    , call4to6 = "4-6"
    , call7to10 = "7-10"
    , call10plus = "10+"

    -- Features (Owner module)
    , feature1Title = "Centralized chat with all your tenants"
    , feature1Desc = "One channel for all exchanges"
    , feature2Title = "Hidden number for your peace of mind"
    , feature2Desc = "Protect your personal number"
    , feature3Title = "Incident tickets with real-time tracking"
    , feature3Desc = "No more lost messages"
    , feature4Title = "Automatic rent receipt sending"
    , feature4Desc = "No more reminders"
    , feature5Title = "Automated payment reminders"
    , feature5Desc = "Respectful but effective"
    , feature6Title = "Bank connection for payment detection"
    , feature6Desc = "Accounting automation"
    , feature7Title = "Marketplace of qualified service providers"
    , feature7Desc = "Quick interventions"
    , feature8Title = "AI-guided diagnostics"
    , feature8Desc = "Solve before calling a pro"
    , feature9Title = "Integrated electronic signature"
    , feature9Desc = "Leases, amendments, inventories"
    }
