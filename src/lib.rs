// Copyright (C) 2026 Jorge Andre Castro
// GPL-2.0-or-later
//! # Corell Kernel
//!
//! `corell` est un noyau d'inférence IA universel, local-first (0 réseau), asynchrone
//! et agnostique des modèles, conçu spécifiquement en Rust pour garantir une souveraineté numérique absolue.
//!
//! Ce framework abstrait la plomberie liée à la communication avec les LLM locaux (via Ollama)
//! et sécurise la persistance des données d'état via un moteur Key-Value embarqué (`sled`).
//!
//! ## Philosophie et Sécurité
//! - **Zéro Réseau :** Aucun octet ne quitte la machine hôte. Idéal pour les environnements médicaux ou juridiques.
//! - **Agnostique :** Basculez d'un modèle à un autre (Llama 3.2, DeepSeek-R1) via une simple configuration logicielle.
//! - **GPL-2.0-or-later :** Garanti comme un bien commun immuable pour le logiciel libre.

use std::path::PathBuf;
use ollama_rs::{
    generation::completion::request::GenerationRequest, 
    Ollama
};
use tokio_stream::StreamExt;

pub mod errors;
use errors::KernelError;

/// Le noyau d'infrastructure principal de Corell.
///
/// Cette structure encapsule le client d'inférence asynchrone et le gestionnaire
/// de stockage persistant local. Elle est conçue pour être thread-safe et réutilisable.
pub struct CorellKernel {
    ollama: Ollama,
    model: String,
    storage: sled::Db,
}

impl CorellKernel {
    /// Initialise une nouvelle instance du Kernel Corell.
    ///
    /// Crée ou ouvre une base de données Key-Value au chemin spécifié et lie
    /// le Kernel à l'instance Ollama s'exécutant sur l'environnement local.
    ///
    /// # Arguments
    /// * `model` - Une chaîne de caractères représentant le modèle local cible (ex: `"llama3.2"`, `"deepseek-r1:8b"`).
    /// * `storage_path` - Un `PathBuf` pointant vers le répertoire de stockage des données locales.
    ///
    /// # Errors
    /// Renvoie une erreur [`KernelError::StorageInitError`] si le répertoire de stockage ne peut pas être ouvert ou créé.
    ///
    /// # Examples
    /// ```no_run
    /// use corell::CorellKernel;
    /// use std::path::PathBuf;
    ///
    /// let storage = PathBuf::from("./vault");
    /// let kernel = CorellKernel::new("llama3.2", storage).unwrap();
    /// ```
    pub fn new(model: &str, storage_path: PathBuf) -> Result<Self, KernelError> {
        let ollama = Ollama::default();
        let storage = sled::open(storage_path)
            .map_err(|e| KernelError::StorageInitError(e.to_string()))?;

        Ok(Self {
            ollama,
            model: model.to_string(),
            storage,
        })
    }

    /// Sauvegarde une donnée textuelle de manière persistante dans le stockage local embarqué.
    ///
    /// Les modifications sont immédiatement poussées sur le disque de manière synchrone via une opération `flush`.
    ///
    /// # Errors
    /// Renvoie une erreur [`KernelError::StorageOperationError`] en cas de défaillance matérielle ou de corruption de l'index.
    pub fn save_data(&self, key: &str, value: &str) -> Result<(), KernelError> {
        self.storage
            .insert(key, value)
            .map_err(|e| KernelError::StorageOperationError(e.to_string()))?;
        self.storage
            .flush()
            .map_err(|e| KernelError::StorageOperationError(e.to_string()))?;
        Ok(())
    }

    /// Récupère une chaîne de caractères stockée localement à partir de sa clé.
    ///
    /// # Errors
    /// Renvoie une erreur [`KernelError::StorageOperationError`] si la lecture échoue ou si les octets récupérés ne forment pas une chaîne UTF-8 valide.
    pub fn get_data(&self, key: &str) -> Result<Option<String>, KernelError> {
        let result = self.storage
            .get(key)
            .map_err(|e| KernelError::StorageOperationError(e.to_string()))?;
        
        match result {
            Some(ivec) => String::from_utf8(ivec.to_vec())
                .map(Some)
                .map_err(|e| KernelError::StorageOperationError(e.to_string())),
            None => Ok(None),
        }
    }

    /// Exécute une inférence universelle synchrone et renvoie le bloc de texte complet généré.
    ///
    /// Le prompt final est automatiquement structuré pour isoler de manière stricte les directives du système
    /// et les données utilisateurs fournies en entrée.
    ///
    /// # Arguments
    /// * `system_prompt` - Les consignes métiers définissant le rôle de l'IA (ex: *"Tu es un médecin légiste..."*).
    /// * `input_data` - Le contenu textuel brut à traiter ou analyser (ex: l'extraction brute d'un dossier clinique).
    ///
    /// # Errors
    /// Renvoie une erreur [`KernelError::InferenceError`] si le démon Ollama ne répond pas ou échoue à traiter la requête.
    pub async fn execute(&self, system_prompt: &str, input_data: &str) -> Result<String, KernelError> {
        let final_prompt = format!("System:\n{}\n\nInput:\n{}", system_prompt, input_data);
        let request = GenerationRequest::new(self.model.clone(), final_prompt);
        
        let response = self.ollama
            .generate(request)
            .await
            .map_err(|e| KernelError::InferenceError(e.to_string()))?;
            
        Ok(response.response)
    }

  /// Exécute une inférence universelle asynchrone en mode **Streaming** (Token-by-Token).
    ///
    /// Cette méthode renvoie un flux (`Stream`) asynchrone permettant de consommer les fragments de texte
    /// en temps réel au fur et à mesure de leur génération par le modèle.
    ///
    /// # Errors
    /// Renvoie une erreur [`KernelError::InferenceError`] dès l'initialisation du flux si la connexion avec le moteur local échoue.
    pub async fn execute_stream(
        &self, 
        system_prompt: &str, 
        input_data: &str
    ) -> Result<impl tokio_stream::Stream<Item = Result<String, KernelError>>, KernelError> {
        let final_prompt = format!("System:\n{}\n\nInput:\n{}", system_prompt, input_data);
        let request = GenerationRequest::new(self.model.clone(), final_prompt);
        
        // On repasse sur la méthode officielle reconnue par ton compilateur
        let stream = self.ollama
            .generate_stream(request)
            .await
            .map_err(|e| KernelError::InferenceError(e.to_string()))?;
            
        // On garde la fusion propre du vecteur de morceaux (chunks)
        Ok(stream.map(|res| {
            res.map(|chunks| {
                chunks.iter()
                    .map(|chunk| chunk.response.clone())
                    .collect::<String>()
            })
            .map_err(|e| KernelError::InferenceError(e.to_string()))
        }))
    }
}