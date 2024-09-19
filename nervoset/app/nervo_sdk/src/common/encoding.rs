/// Base64 encoding/decoding
pub mod base64 {
    use base64::alphabet;
    use base64::engine::{general_purpose, GeneralPurpose};
    use serde_derive::{Deserialize, Serialize};

    const URL_SAFE_ENGINE: GeneralPurpose =
        GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Base64Text(pub String);

    impl Base64Text {
        pub fn text(self) -> String {
            self.0.clone()
        }
    }

    impl From<&Base64Text> for String {
        fn from(base64: &Base64Text) -> Self {
            let Base64Text(base64_text) = base64;
            base64_text.clone()
        }
    }

    pub mod encoder {
        use crate::common::encoding::base64::{Base64Text, URL_SAFE_ENGINE};
        use base64::Engine as _;

        impl From<Vec<u8>> for Base64Text {
            fn from(data: Vec<u8>) -> Self {
                Base64Text::from(data.as_slice())
            }
        }

        impl From<&[u8]> for Base64Text {
            fn from(data: &[u8]) -> Self {
                Self(URL_SAFE_ENGINE.encode(data))
            }
        }

        impl From<String> for Base64Text {
            fn from(data: String) -> Self {
                Base64Text::from(data.as_bytes())
            }
        }

        impl From<&str> for Base64Text {
            fn from(data: &str) -> Self {
                Base64Text::from(data.as_bytes())
            }
        }
    }

    pub mod decoder {
        use crate::common::encoding::base64::{Base64Text, URL_SAFE_ENGINE};
        use base64::Engine as _;

        impl TryFrom<&Base64Text> for Vec<u8> {
            type Error = anyhow::Error;

            fn try_from(base64: &Base64Text) -> Result<Self, Self::Error> {
                let Base64Text(base64_text) = base64;
                let data = URL_SAFE_ENGINE.decode(base64_text)?;
                Ok(data)
            }
        }
    }

    #[cfg(test)]
    mod test {
        use crate::common::encoding::base64::Base64Text;

        const TEST_STR: &str = "kjsfdbkjsfhdkjhsfdkjhsfdkjhksfdjhksjfdhksfd";
        const ENCODED_URL_SAFE_TEST_STR: &str =
            "a2pzZmRia2pzZmhka2poc2Zka2poc2Zka2poa3NmZGpoa3NqZmRoa3NmZA";

        #[test]
        fn from_vec() {
            let encoded = Base64Text::from(vec![65, 65, 65]);
            let expected = Base64Text("QUFB".to_string());
            assert_eq!(encoded, expected);
        }

        #[test]
        fn from_bytes() {
            let encoded = Base64Text::from(TEST_STR.as_bytes());
            let expected = Base64Text(ENCODED_URL_SAFE_TEST_STR.to_string());
            assert_eq!(encoded, expected);
        }
    }
}
