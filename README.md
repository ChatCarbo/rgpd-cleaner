# 🛡️ RGPD Cleaner

> **Reprenez le contrôle de vos données. Découvrez où sont vos comptes en ligne et préparez vos demandes d'effacement (Art. 17 RGPD) localement, gratuitement et sans tracker.**

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.10%2B-blue?logo=python)](https://www.python.org/)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)
[![GDPR](https://img.shields.io/badge/GDPR-Compliant-purple)](https://gdpr-info.eu/)

---

## 🎯 Pourquoi ce projet ?

En 2025, un internaute moyen possède **plus de 100 comptes en ligne**, souvent oubliés. Faire valoir son droit à l'effacement est un parcours du combattant. 

**RGPD Cleaner part d'un principe simple :** votre droit à l'effacement est **gratuit**, **légal**, et **vous pouvez l'exercer vous-même**.

Inspiré par la philosophie des créateurs de contenu qui nous apprennent à ne plus se faire avoir sur internet (merci [@madameb0nplan](https://www.instagram.com/madameb0nplan/)), cet outil vous donne les clés pour faire le ménage, sans payer.

---

## ✨ Fonctionnalités

- **🔍 Détection automatique** : Utilise [`holehe`](https://github.com/megadose/holehe) pour tester votre adresse e-mail sur plus de 120 plateformes en quelques secondes.
- **🧠 Classification intelligente** : Croise les résultats avec la base [JustDeleteMe](https://justdeleteme.xyz/) pour savoir **exactement** comment supprimer chaque compte :
  - 🟢 **Direct** : Un bouton "Supprimer" existe sur le site.
  - 🟡 **Formulaire** : Nécessite de remplir un formulaire RGPD.
  - 🔵 **Email** : Nécessite l'envoi d'un email (l'outil génère le courrier).
  - 🟠 **Ticket** : Nécessite d'ouvrir un ticket support.
  - 🔴 **Impossible** : Le site refuse la suppression.
- **📝 Génération de courriers RGPD** : Pour les sites qui exigent un email, l'outil génère un courrier prêt à l'emploi citant l'Article 17 et menaçant de saisir la CNIL.
- **🇫🇷 / 🇬🇧 Bilingue** : Interface disponible en Français et en Anglais (détection automatique).
- **🔒 100% Local & Privé** : Aucune donnée n'est envoyée. Pas de compte à créer. Pas de télémétrie.

---

## 🏗️ Architecture technique

Le projet est divisé en deux parties pour allier performance et flexibilité :

- **Frontend (Rust + `egui`)** : Interface graphique native, ultra-rapide et légère. Gère l'affichage, les boutons et l'ouverture des liens.
- **Backend (Python)** : Moteur de scan qui orchestre `holehe`, lit la base de données `sites.json` et génère les fichiers textes.

Ils communiquent en temps réel via `stdout` en utilisant le format **JSON Lines**.

---

## 🚀 Installation & Utilisation

### Prérequis
- [Rust](https://rustup.rs/) (1.75+)
- [Python](https://www.python.org/) (3.10+)
- `holehe` : `pip install holehe`

### Compilation

```bash
# 1. Cloner le dépôt
git clone https://github.com/ChatCarbo/rgpd-cleaner.git
cd rgpd-cleaner

# 2. Compiler le moteur Python en exécutable (Windows)
pyinstaller --onefile engine.py
# (Sur Mac/Linux, gardez engine.py tel quel)

# 3. Compiler l'interface Rust
cargo build --release
