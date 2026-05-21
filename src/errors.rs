// Copyright (C) 2026 Jorge Andre Castro
// GPL-2.0-or-later
//! # Gestion des Erreurs Système pour le Kernel Corell
//!
//! Ce module centralise et formalise l'intégralité des pannes et anomalies
//! susceptibles de survenir durant le cycle de vie du noyau d'inférence.
//! Il s'appuie sur la crate `thiserror` pour générer des messages d'erreur clairs et explicites.

use thiserror::Error;

/// Énumération descriptive des erreurs d'infrastructure du Kernel.
#[derive(Error, Debug)]
pub enum KernelError {
    /// Levée lorsque le sous-système de stockage persistant ne peut pas s'initialiser.
    /// Cela se produit généralement en cas de droits d'accès insuffisants sur le répertoire cible.
    #[error("Erreur critique d'initialisation du stockage embarqué Sled : {0}")]
    StorageInitError(String),

    /// Levée lors d'un échec d'écriture (`insert`), de lecture (`get`) ou de synchronisation synchrone (`flush`) dans la base locale.
    #[error("Erreur d'opération sur le stockage local : {0}")]
    StorageOperationError(String),

    /// Levée lorsque le démon Ollama local est inaccessible, non lancé, ou si le modèle demandé n'a pas été téléchargé au préalable.
    #[error("Échec de la communication avec le pilote Ollama local : {0}")]
    InferenceError(String),

    /// Erreur générique d'Entrée/Sortie renvoyée par le système d'exploitation hôte.
    #[error("Erreur d'E/S système : {0}")]
    IoError(#[from] std::io::Error),
}