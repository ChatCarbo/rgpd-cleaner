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
- **📝 Génération de courriers RGPD** : Pour les sites qui exigent un email, l'outil génère un courrier prêt à l'emploi citant l'Article 17 du RGPD et mentionnant un signalement CNIL en l'absence de réponse.
- **🇫🇷 / 🇬🇧 Bilingue** : Interface disponible en Français et en Anglais (détection automatique).
- **🔒 100% Local & Privé** : Aucune donnée n'est envoyée vers un serveur externe. Pas de compte à créer. Pas de télémétrie.

---

## 🏗️ Architecture technique

Le projet est divisé en deux briques complémentaires pour allier performance et flexibilité :

- **Frontend (Rust + `egui`)** : Interface graphique native, ultra-rapide et légère. Gère l'affichage, la navigation, les filtres et l'ouverture des liens.
- **Backend (Python)** : Moteur de scan (`engine.py`) qui orchestre `holehe`, lit la base de données locale `sites.json` et génère les modèles de courriers textuels.

L'interface Rust et le backend Python communiquent en temps réel via `stdout` en utilisant le format **JSON Lines**.

---

## 🚀 Installation & Build

### Prérequis
- [Rust](https://rustup.rs/) (1.75 ou supérieur)
- [Python](https://www.python.org/) (3.10 ou supérieur)
- `pip` pour l'installation des dépendances Python

### 1. Préparation de l'environnement

```bash
# Cloner le dépôt
git clone https://github.com/ChatCarbo/rgpd-cleaner.git
cd rgpd-cleaner

# Installer les dépendances Python
pip install holehe pyinstaller
```

### 2. Compilation du moteur Backend (Python)

```bash
# Sous Windows : compiler le script Python en binaire autonome
pyinstaller --onefile engine.py

# (Sur Linux/macOS, vous pouvez conserver engine.py à la racine du projet)
```

### 3. Compilation du Frontend (Rust)

```bash
# Compiler l'application Rust en mode release
cargo build --release
```

Le binaire exécutable sera généré dans le dossier `target/release/`.

---

## 💡 Utilisation

1. Placez les fichiers `sites.json` et `contacts.json` (téléchargeables depuis JustDeleteMe / inclus dans le projet) dans le même répertoire que l'exécutable.
2. Lancez l'application (`rgpd-cleaner.exe` ou `./rgpd-cleaner`).
3. Saisissez votre adresse e-mail.
4. Cochez la case de certification.
5. Cliquez sur **Scanner**.
6. Filtrez les résultats par méthode de suppression (conseil : commencez par les 🟢 **Directs** !).
7. Suivez les instructions affichées ou récupérez les courriers `.txt` générés localement pour les envoyer aux DPO des services concernés.

---

## 🔐 Privacy by Design

Ce projet applique à lui-même les principes qu'il aide à faire respecter :

- **Minimisation des données** : Les fichiers temporaires générés par `holehe` sont immédiatement supprimés après lecture (gestion par `atexit` et `SIGTERM`).
- **Aucune automatisation d'envoi** : L'application produit des fichiers `.txt` locaux. C'est vous qui envoyez l'e-mail depuis votre propre boîte messagerie pour garantir la traçabilité juridique.
- **Transparence totale** : Le code est 100 % open-source et auditable.

---

## 📜 Licence

Ce projet est sous licence **MIT**. Consultez le fichier [LICENSE](LICENSE) pour plus de détails.

---

## 🙏 Remerciements

- [JustDeleteMe](https://justdeleteme.xyz/) pour leur base de données.
- [holehe](https://github.com/megadose/holehe) pour le moteur de détection de comptes.
- [egui](https://github.com/emilk/egui) pour le framework GUI Rust.