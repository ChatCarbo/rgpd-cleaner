#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Fr,
    En,
}

impl Lang {
    pub fn detect() -> Self {
        let lang_var = std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .or_else(|_| std::env::var("LC_MESSAGES"))
            .or_else(|_| std::env::var("LANGUAGE"))
            .unwrap_or_default()
            .to_lowercase();

        #[cfg(target_os = "windows")]
        {
            if lang_var.starts_with("en") { return Lang::En; }
            if let Ok(output) = std::process::Command::new("powershell")
                .args(["-Command", "(Get-Culture).Name"]).output()
            {
                let s = String::from_utf8_lossy(&output.stdout).to_lowercase();
                if s.starts_with("en") { return Lang::En; }
            }
            return Lang::Fr;
        }

        #[cfg(not(target_os = "windows"))]
        {
            if lang_var.starts_with("en") { Lang::En } else { Lang::Fr }
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Lang::Fr => "🇫🇷 Français",
            Lang::En => "🇬🇧 English",
        }
    }
}

pub struct Strings {
    pub app_title: &'static str,
    pub tagline: &'static str,
    pub reminder: &'static str,
    pub email_label: &'static str,
    pub name_label: &'static str,
    pub python_label: &'static str,
    pub certify: &'static str,
    pub language_label: &'static str,
    pub advanced_settings: &'static str,
    pub python_help: &'static str,
    pub scan_button: &'static str,
    pub scanning: &'static str,
    pub open_folder: &'static str,
    pub log_title: &'static str,
    pub accounts_title: &'static str,
    pub filter_label: &'static str,
    pub filter_all: &'static str,
    pub filter_direct: &'static str,
    pub filter_form: &'static str,
    pub filter_email: &'static str,
    pub filter_ticket: &'static str,
    pub filter_impossible: &'static str,
    pub col_site: &'static str,
    pub col_method: &'static str,
    pub col_contact: &'static str,
    pub col_reliability: &'static str,
    pub col_action: &'static str,
    pub requires_login_tip: &'static str,
    pub done_message: &'static str,
    pub stopped_message: &'static str,
    pub launching_error: &'static str,
    pub engine_stopped_no_result: &'static str,
    pub save_dialog_title: &'static str,
    pub file_prefix: &'static str,
}

pub const FR: Strings = Strings {
    app_title: "RGPD Cleaner",
    tagline: "Retrouve les comptes liés à TON adresse e-mail et prépare des demandes d'effacement (art. 17 RGPD). Rien n'est envoyé : tu relis et tu agis toi-même.",
    reminder: "⚠️ Rappel : Vous devez impérativement utiliser le courrier téléchargé depuis l'adresse e-mail saisie ci-dessous.",
    email_label: "Adresse e-mail",
    name_label: "Prénom Nom",
    python_label: "Commande Python",
    certify: "Je certifie que cette adresse est la mienne",
    language_label: "Langue :",
    advanced_settings: "⚙️ Paramètres avancés",
    python_help: "Commande pour lancer Python (ex: python, py, python3, ou chemin complet)",
    scan_button: "Scanner et préparer les brouillons",
    scanning: "Scan en cours...",
    open_folder: "Ouvrir le dossier",
    log_title: "Journal",
    accounts_title: "Comptes trouvés",
    filter_label: "Filtre :",
    filter_all: "Tous",
    filter_direct: "🟢 Direct",
    filter_form: "🟡 Formulaire",
    filter_email: "🔵 Email",
    filter_ticket: "🟠 Ticket",
    filter_impossible: "🔴 Impossible",
    col_site: "Site",
    col_method: "Méthode",
    col_contact: "Contact RGPD",
    col_reliability: "Fiabilité",
    col_action: "Action",
    requires_login_tip: "🔒 Connexion requise.",
    done_message: "Terminé : {} brouillon(s) prêt(s). Rien n'a été envoyé.",
    stopped_message: "Scan interrompu par l'utilisateur.",
    launching_error: "Impossible de lancer '{}' : {}",
    engine_stopped_no_result: "Le moteur s'est arrêté sans résultat (code {:?}).",
    save_dialog_title: "Enregistrer le brouillon",
    file_prefix: "demande_suppression_",
};

pub const EN: Strings = Strings {
    app_title: "RGPD Cleaner",
    tagline: "Find every account linked to YOUR email address and prepare GDPR erasure requests (Art. 17). Nothing is sent: you review and act yourself.",
    reminder: "⚠️ Reminder: You must absolutely send the downloaded letter from the email address entered below.",
    email_label: "Email address",
    name_label: "First Last name",
    python_label: "Python command",
    certify: "I certify that this address is mine",
    language_label: "Language:",
    advanced_settings: "⚙️ Advanced settings",
    python_help: "Command to launch Python (e.g. python, py, python3, or full path)",
    scan_button: "Scan and prepare drafts",
    scanning: "Scan in progress...",
    open_folder: "Open folder",
    log_title: "Log",
    accounts_title: "Accounts found",
    filter_label: "Filter:",
    filter_all: "All",
    filter_direct: "🟢 Direct",
    filter_form: "🟡 Form",
    filter_email: "🔵 Email",
    filter_ticket: "🟠 Ticket",
    filter_impossible: "🔴 Impossible",
    col_site: "Site",
    col_method: "Method",
    col_contact: "GDPR contact",
    col_reliability: "Reliability",
    col_action: "Action",
    requires_login_tip: "🔒 Login required.",
    done_message: "Done: {} draft(s) ready. Nothing has been sent.",
    stopped_message: "Scan interrupted by user.",
    launching_error: "Unable to launch '{}': {}",
    engine_stopped_no_result: "Engine stopped without results (code {:?}).",
    save_dialog_title: "Save draft",
    file_prefix: "deletion_request_",
};

impl Strings {
    pub fn for_lang(lang: Lang) -> &'static Strings {
        match lang {
            Lang::Fr => &FR,
            Lang::En => &EN,
        }
    }
}