// app-remoto: extensões do hbbs oficial, isoladas aqui para facilitar a sincronização com o original.
//
// 1. Troca de chaves no TCP (KeyExchange): o cliente RustDesk logado exige um canal cifrado com o
//    servidor antes de enviar o token. O hbbs oficial não implementa; sem isso a conexão falha com
//    "Failed to secure tcp: deadline has elapsed".
// 2. Login obrigatório (MUST_LOGIN=Y): só repassa pedidos de conexão com token válido, assinado
//    (HS256) pela API da aplicação com o mesmo segredo (RUSTDESK_API_JWT_KEY).

use crate::common::get_arg;
use hbb_common::{
    bytes::Bytes,
    log,
    rendezvous_proto::{KeyExchange, RendezvousMessage},
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use once_cell::sync::Lazy;
use serde_derive::Deserialize;
use sodiumoxide::crypto::{box_, secretbox, sign};

static MUST_LOGIN: Lazy<bool> = Lazy::new(|| get_arg("MUST_LOGIN").to_uppercase() == "Y");
static JWT_SECRET: Lazy<String> = Lazy::new(|| get_arg("RUSTDESK_API_JWT_KEY"));

/// Par de chaves do servidor para a troca de chaves (gerado na inicialização).
pub struct SecureTcpKeys {
    pub pk: box_::PublicKey,
    pub sk: box_::SecretKey,
}

impl SecureTcpKeys {
    pub fn generate() -> Self {
        let (pk, sk) = box_::gen_keypair();
        Self { pk, sk }
    }
}

pub fn log_config() {
    log::info!("MUST_LOGIN={}", if *MUST_LOGIN { "Y" } else { "N" });
    if *MUST_LOGIN && JWT_SECRET.is_empty() {
        log::warn!("MUST_LOGIN=Y sem RUSTDESK_API_JWT_KEY: todos os pedidos de conexão serão recusados");
    }
}

/// Fase 1: envia a chave pública temporária do servidor, assinada com a chave do servidor.
pub fn key_exchange_phase1(keys: &SecureTcpKeys, server_sk: &sign::SecretKey) -> RendezvousMessage {
    let signed = sign::sign(&keys.pk.0, server_sk);
    let mut msg = RendezvousMessage::new();
    msg.set_key_exchange(KeyExchange {
        keys: vec![Bytes::from(signed)],
        ..Default::default()
    });
    msg
}

/// Fase 2: abre a chave simétrica enviada pelo cliente. Mensagem inválida devolve None
/// (a conexão é encerrada; nunca derruba o processo).
pub fn key_exchange_phase2(keys: &SecureTcpKeys, ex: &KeyExchange) -> Option<secretbox::Key> {
    if ex.keys.len() != 2 {
        log::warn!("KeyExchange com {} chaves (esperado 2)", ex.keys.len());
        return None;
    }
    let their_pk = box_::PublicKey::from_slice(&ex.keys[0])?;
    let nonce = box_::Nonce([0u8; box_::NONCEBYTES]);
    let opened = box_::open(&ex.keys[1], &nonce, &their_pk, &keys.sk).ok()?;
    secretbox::Key::from_slice(&opened)
}

#[derive(Deserialize)]
struct Claims {
    #[allow(dead_code)]
    user_id: u64,
}

/// Login obrigatório: None = liberado; Some(motivo) = recusar o pedido de conexão.
pub fn check_login(token: &str) -> Option<&'static str> {
    if !*MUST_LOGIN {
        return None;
    }
    if token.is_empty() {
        return Some("Faça login no Console para conectar");
    }
    if JWT_SECRET.is_empty() {
        return Some("Servidor sem RUSTDESK_API_JWT_KEY configurado");
    }
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    match decode::<Claims>(token, &DecodingKey::from_secret(JWT_SECRET.as_bytes()), &validation) {
        Ok(_) => None,
        Err(e) => {
            log::debug!("token recusado: {e}");
            Some("Login expirado. Saia e entre de novo no Console")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn troca_de_chaves_completa() {
        sodiumoxide::init().ok();
        let server = SecureTcpKeys::generate();
        let (server_sign_pk, server_sign_sk) = sign::gen_keypair();
        // fase 1: cliente confere a assinatura e obtém a chave pública temporária
        let m1 = key_exchange_phase1(&server, &server_sign_sk);
        let signed = &m1.key_exchange().keys[0];
        let pk_bytes = sign::verify(signed, &server_sign_pk).expect("assinatura");
        let server_pk = box_::PublicKey::from_slice(&pk_bytes).unwrap();
        // fase 2: cliente sela uma chave simétrica para o servidor
        let (client_pk, client_sk) = box_::gen_keypair();
        let sym = secretbox::gen_key();
        let nonce = box_::Nonce([0u8; box_::NONCEBYTES]);
        let sealed = box_::seal(&sym.0, &nonce, &server_pk, &client_sk);
        let ex = KeyExchange { keys: vec![Bytes::from(client_pk.0.to_vec()), Bytes::from(sealed)], ..Default::default() };
        assert_eq!(key_exchange_phase2(&server, &ex).unwrap().0, sym.0);
    }

    #[test]
    fn mensagem_invalida_nao_derruba() {
        let server = SecureTcpKeys::generate();
        let ex = KeyExchange { keys: vec![Bytes::from(vec![1u8; 32]), Bytes::from(vec![2u8; 48])], ..Default::default() };
        assert!(key_exchange_phase2(&server, &ex).is_none());
        let curto = KeyExchange { keys: vec![Bytes::from(vec![1u8; 3])], ..Default::default() };
        assert!(key_exchange_phase2(&server, &curto).is_none());
    }
}
