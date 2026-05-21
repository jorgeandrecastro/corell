# corell

> **Un kernel d'inférence IA universel, asynchrone, agnostique et local-first (0 réseau) en Rust.**

[![Crates.io](https://img.shields.io/crates/v/corell)](https://crates.io/crates/corell)
[![Docs.rs](https://docs.rs/corell/badge.svg)](https://docs.rs/corell)
[![License: GPL-2.0-or-later](https://img.shields.io/badge/license-GPL--2.0--or--later-blue)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.html)
[![Rust Edition 2024](https://img.shields.io/badge/rust-2024-orange)](https://doc.rust-lang.org/edition-guide/rust-2024/)

---

## Philosophie

`corell` est conçu pour un seul impératif : **aucun octet ne quitte votre machine**.

Dans les environnements où la souveraineté des données est non négociable  dossiers médicaux, documents juridiques, propriété intellectuelle industrielle  les solutions cloud ne sont pas une option. `corell` abstrait la plomberie d'inférence locale via [Ollama](https://ollama.com) et sécurise la persistance d'état via un moteur Key-Value embarqué ([sled](https://github.com/spacejam/sled)), le tout sans aucune dépendance réseau externe.

**Trois garanties fondamentales :**

- **Zéro réseau** : Conçu pour des environnements air-gapped. Aucune télémétrie, aucun appel sortant.
- **Agnosticisme modèle** : Basculez de `llama3.2` à `deepseek-r1:8b` ou tout modèle Ollama via une simple chaîne de caractères.
- **Bien commun** : Licencié GPL-2.0-or-later. Immuablement libre.

---

## Prérequis

1. **Rust ≥ 1.85** (édition 2024 requise)
2. **[Ollama](https://ollama.com/download)** installé et en cours d'exécution sur `localhost:11434`
3. Le modèle cible téléchargé au préalable :

```sh
ollama pull llama3.2
# ou
ollama pull deepseek-r1:8b
```

---

## Installation

Ajoutez `corell` à votre `Cargo.toml` :

```toml
[dependencies]
corell = "0.1"
tokio = { version = "1", features = ["full"] }
```

---

## Démarrage rapide

### Inférence synchrone (réponse complète)

```rust
use corell::CorellKernel;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kernel = CorellKernel::new("llama3.2", PathBuf::from("./vault"))?;

    let response = kernel
        .execute(
            "Tu es un expert en résumé juridique. Réponds en français.",
            "Analyse ce contrat : [...]",
        )
        .await?;

    println!("{}", response);
    Ok(())
}
```

### Inférence en streaming (token par token)

```rust
use corell::CorellKernel;
use std::path::PathBuf;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kernel = CorellKernel::new("llama3.2", PathBuf::from("./vault"))?;

    let mut stream = kernel
        .execute_stream(
            "Tu es un assistant médical. Ne fournis pas de diagnostic.",
            "Quels sont les symptômes courants d'une carence en vitamine D ?",
        )
        .await?;

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(text) => print!("{}", text),
            Err(e) => eprintln!("Erreur de streaming : {}", e),
        }
    }
    Ok(())
}
```

### Persistance locale (Key-Value)

```rust
use corell::CorellKernel;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kernel = CorellKernel::new("llama3.2", PathBuf::from("./vault"))?;

    // Écriture
    kernel.save_data("session:001:context", "Résumé de la session précédente...")?;

    // Lecture
    if let Some(value) = kernel.get_data("session:001:context")? {
        println!("Contexte récupéré : {}", value);
    }

    Ok(())
}
```

---

## Architecture

```
corell/
├── src/
│   ├── lib.rs          # CorellKernel : point d'entrée public
│   └── errors.rs       # KernelError  : énumération d'erreurs typées
├── Cargo.toml
└── README.md
```

### `CorellKernel`

| Méthode | Description |
|---|---|
| `new(model, path)` | Initialise le kernel et ouvre/crée le stockage local |
| `execute(system, input)` | Inférence bloquante, retourne le texte complet |
| `execute_stream(system, input)` | Inférence en streaming, retourne un `Stream<Item = Result<String, KernelError>>` |
| `save_data(key, value)` | Persiste une valeur avec flush synchrone sur disque |
| `get_data(key)` | Récupère une valeur par clé (`Option<String>`) |

### `KernelError`

| Variante | Cause |
|---|---|
| `StorageInitError` | Droits insuffisants sur le répertoire de stockage |
| `StorageOperationError` | Échec lecture/écriture/flush dans sled |
| `InferenceError` | Ollama inaccessible ou modèle non téléchargé |
| `IoError` | Erreur d'E/S système (via `#[from] std::io::Error`) |

---

## Modèles testés

| Modèle | Commande pull | Usage recommandé |
|---|---|---|
| `llama3.2` | `ollama pull llama3.2` | Usage général, contexte long |
| `deepseek-r1:8b` | `ollama pull deepseek-r1:8b` | Raisonnement, analyse |
| `mistral` | `ollama pull mistral` | Polyvalent, rapide |
| `phi4` | `ollama pull phi4` | Machines à ressources limitées |

Tout modèle compatible Ollama fonctionne : passez simplement son identifiant à `CorellKernel::new`.

---

## Cas d'usage

`corell` est particulièrement adapté pour :

- **Systèmes de santé** : Analyse de dossiers cliniques sans quitter le SI hospitalier
- **Cabinets juridiques** : Résumé et extraction d'informations contractuelles en circuit fermé
- **Industrie et R&D** : Traitement de données sensibles soumises à des accords de confidentialité
- **Administrations** : Respect des réglementations sur la localisation des données (RGPD, HDS)
- **Développeurs** : Couche d'abstraction légère pour tout projet Rust nécessitant de l'IA locale

---

## Feuille de route

- [ ] `new_with_host(model, host, port, path)` : Ollama sur hôte ou port personnalisé
- [ ] Templates de prompt configurables (Alpaca, ChatML, Llama-3)
- [ ] Gestion du contexte conversationnel multi-tours
- [ ] Suppression de clé dans le stockage (`delete_data`)
- [ ] Chiffrement optionnel du stockage sled (via `age`)

---

## Contribution

Les contributions sont les bienvenues. Ce projet suit les principes du logiciel libre.

```sh
git clone https://github.com/jorgeandrecastro/corell
cd corell
cargo build
cargo test
```

Merci d'ouvrir une *issue* avant toute PR substantielle afin d'en discuter le périmètre.

---

## Licence

`corell` est distribué sous licence **GPL-2.0-or-later**.

Vous êtes libre de l'utiliser, le modifier et le redistribuer, à condition que tout travail dérivé reste sous la même licence. Voir [LICENSE](./LICENSE) pour le texte complet.

---

Développé par **Jorge Andre Castro**. 

