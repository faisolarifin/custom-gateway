use webhook_gateway::utils::generate_signature;

#[test]
fn test_generate_signature() {
    let permata_static_key = "permata_static_key";
    let key = "secret_key";
    let timestamp = "1634567890";
    let data = "test_data";
    
    let signature = generate_signature(permata_static_key, key, timestamp, data).unwrap();
    assert!(!signature.is_empty());
    
    let same_signature = generate_signature(permata_static_key, key, timestamp, data).unwrap();
    assert_eq!(signature, same_signature);
}

#[test]
fn test_different_inputs_different_signatures() {
    let permata_static_key = "permata_static_key";
    let key = "secret_key";
    let timestamp1 = "1634567890";
    let timestamp2 = "1634567891";
    let data = "test_data";
    
    let signature1 = generate_signature(permata_static_key, key, timestamp1, data).unwrap();
    let signature2 = generate_signature(permata_static_key, key, timestamp2, data).unwrap();
    
    assert_ne!(signature1, signature2);
}

#[test]
fn test_signature_consistency() {
    let permata_static_key = "permata_static_key";
    let key = "my_secret";
    let timestamp = "1609459200";
    let data = "hello world";

    let signature = generate_signature(permata_static_key, key, timestamp, data).unwrap();
    
    for _ in 0..10 {
        let new_signature = generate_signature(permata_static_key, key, timestamp, data).unwrap();
        assert_eq!(signature, new_signature, "Signature should be consistent for same inputs");
    }
}