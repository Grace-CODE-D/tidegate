#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};

const MAX_ENDPOINT_LENGTH: u32 = 256;
const MAX_VERSION_LENGTH: u32 = 64;

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Gateway {
    pub operator: Address,
    pub endpoint: String,
    pub version: String,
    pub active: bool,
}

#[derive(Clone)]
#[contracttype]
enum DataKey {
    Gateway(Address),
}

#[contract]
pub struct GatewayRegistry;

#[contractimpl]
impl GatewayRegistry {
    pub fn register(env: Env, operator: Address, endpoint: String, version: String) -> Gateway {
        operator.require_auth();
        validate_text(&endpoint, MAX_ENDPOINT_LENGTH);
        validate_text(&version, MAX_VERSION_LENGTH);

        let key = DataKey::Gateway(operator.clone());
        if env.storage().persistent().has(&key) {
            panic!("gateway is already registered");
        }

        let gateway = Gateway {
            operator,
            endpoint,
            version,
            active: true,
        };
        env.storage().persistent().set(&key, &gateway);
        gateway
    }

    pub fn update(env: Env, operator: Address, endpoint: String, version: String) -> Gateway {
        operator.require_auth();
        validate_text(&endpoint, MAX_ENDPOINT_LENGTH);
        validate_text(&version, MAX_VERSION_LENGTH);

        let key = DataKey::Gateway(operator.clone());
        let mut gateway: Gateway = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic!("gateway is not registered"));
        gateway.endpoint = endpoint;
        gateway.version = version;
        gateway.active = true;
        env.storage().persistent().set(&key, &gateway);
        gateway
    }

    pub fn deactivate(env: Env, operator: Address) {
        operator.require_auth();
        let key = DataKey::Gateway(operator);
        let mut gateway: Gateway = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic!("gateway is not registered"));
        gateway.active = false;
        env.storage().persistent().set(&key, &gateway);
    }

    pub fn get(env: Env, operator: Address) -> Option<Gateway> {
        env.storage().persistent().get(&DataKey::Gateway(operator))
    }
}

fn validate_text(value: &String, max_length: u32) {
    if value.is_empty() || value.len() > max_length {
        panic!("text value has an invalid length");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    #[test]
    fn operator_can_register_read_update_and_deactivate() {
        let env = Env::default();
        let contract_id = env.register(GatewayRegistry, ());
        let client = GatewayRegistryClient::new(&env, &contract_id);
        let operator = Address::generate(&env);
        let endpoint = String::from_str(&env, "https://testnet.example/gateway");
        let version = String::from_str(&env, "0.1.0");

        env.mock_all_auths();
        let registered = client.register(&operator, &endpoint, &version);
        assert_eq!(registered.operator, operator);
        assert!(registered.active);
        assert_eq!(client.get(&operator), Some(registered.clone()));

        let updated_endpoint = String::from_str(&env, "https://testnet.example/v2");
        let updated = client.update(&operator, &updated_endpoint, &version);
        assert_eq!(updated.endpoint, updated_endpoint);
        assert!(updated.active);

        client.deactivate(&operator);
        assert!(!client.get(&operator).unwrap().active);
    }

    #[test]
    #[should_panic(expected = "text value has an invalid length")]
    fn empty_endpoint_is_rejected() {
        let env = Env::default();
        let contract_id = env.register(GatewayRegistry, ());
        let client = GatewayRegistryClient::new(&env, &contract_id);
        let operator = Address::generate(&env);

        env.mock_all_auths();
        client.register(
            &operator,
            &String::from_str(&env, ""),
            &String::from_str(&env, "0.1.0"),
        );
    }
}
