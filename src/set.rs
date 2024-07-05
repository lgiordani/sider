use crate::storage_result::{StorageError, StorageResult};

#[derive(Debug, PartialEq)]
pub enum KeyExistence {
    NX,
    XX,
}

#[derive(Debug, PartialEq)]
pub enum KeyExpiry {
    EX(u64),
    PX(u64),
}

#[derive(Debug, PartialEq)]
pub struct SetArgs {
    pub expiry: Option<KeyExpiry>,
    pub existence: Option<KeyExistence>,
    pub get: bool,
}

impl SetArgs {
    pub fn new() -> Self {
        SetArgs {
            expiry: None,
            existence: None,
            get: false,
        }
    }
}

pub fn parse_set_arguments(arguments: &Vec<String>) -> StorageResult<SetArgs> {
    let mut args = SetArgs::new();

    let mut idx: usize = 0;

    loop {
        if idx >= arguments.len() {
            break;
        }

        match arguments[idx].to_lowercase().as_str() {
            "nx" => {
                if args.existence == Some(KeyExistence::XX) {
                    return Err(StorageError::CommandSyntaxError(arguments.join(" ")));
                }

                args.existence = Some(KeyExistence::NX);

                idx += 1;
            }
            "xx" => {
                if args.existence == Some(KeyExistence::NX) {
                    return Err(StorageError::CommandSyntaxError(arguments.join(" ")));
                }

                args.existence = Some(KeyExistence::XX);

                idx += 1;
            }
            "get" => {
                args.get = true;
                idx += 1;
            }
            "ex" => {
                if let Some(KeyExpiry::PX(_)) = args.expiry {
                    return Err(StorageError::CommandSyntaxError(arguments.join(" ")));
                }

                if idx + 1 == arguments.len() {
                    return Err(StorageError::CommandSyntaxError(arguments.join(" ")));
                }

                let value: u64 = arguments[idx + 1]
                    .parse()
                    .map_err(|_| StorageError::CommandSyntaxError(arguments.join(" ")))?;

                args.expiry = Some(KeyExpiry::EX(value));

                idx += 2;
            }
            "px" => {
                if let Some(KeyExpiry::EX(_)) = args.expiry {
                    return Err(StorageError::CommandSyntaxError(arguments.join(" ")));
                }

                if idx + 1 == arguments.len() {
                    return Err(StorageError::CommandSyntaxError(arguments.join(" ")));
                }

                let value: u64 = arguments[idx + 1]
                    .parse()
                    .map_err(|_| StorageError::CommandSyntaxError(arguments.join(" ")))?;

                args.expiry = Some(KeyExpiry::PX(value));

                idx += 2;
            }
            _ => {
                return Err(StorageError::CommandSyntaxError(arguments.join(" ")));
            }
        }
    }

    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nx() {
        let commands: Vec<String> = vec![String::from("NX")];

        let args = parse_set_arguments(&commands).unwrap();

        assert_eq!(args.existence, Some(KeyExistence::NX));
    }

    #[test]
    fn test_parse_nx_lowercase() {
        let commands: Vec<String> = vec![String::from("nx")];

        let args = parse_set_arguments(&commands).unwrap();

        assert_eq!(args.existence, Some(KeyExistence::NX));
    }

    #[test]
    fn test_parse_xx() {
        let commands: Vec<String> = vec![String::from("XX")];

        let args = parse_set_arguments(&commands).unwrap();

        assert_eq!(args.existence, Some(KeyExistence::XX));
    }

    #[test]
    fn test_parse_xx_and_nx() {
        let commands: Vec<String> = vec![String::from("XX"), String::from("NX")];

        assert!(matches!(
            parse_set_arguments(&commands),
            Err(StorageError::CommandSyntaxError(_))
        ));
    }

    #[test]
    fn test_parse_nx_and_xx() {
        let commands: Vec<String> = vec![String::from("NX"), String::from("XX")];

        assert!(matches!(
            parse_set_arguments(&commands),
            Err(StorageError::CommandSyntaxError(_))
        ));
    }

    #[test]
    fn test_parse_get() {
        let commands: Vec<String> = vec![String::from("GET")];

        let args = parse_set_arguments(&commands).unwrap();

        assert!(args.get);
    }

    #[test]
    fn test_parse_nx_and_get() {
        let commands: Vec<String> = vec![String::from("NX"), String::from("GET")];

        let args = parse_set_arguments(&commands).unwrap();

        assert_eq!(
            args,
            SetArgs {
                existence: Some(KeyExistence::NX),
                expiry: None,
                get: true
            }
        );
    }

    #[test]
    fn test_parse_xx_and_get() {
        let commands: Vec<String> = vec![String::from("XX"), String::from("GET")];

        let args = parse_set_arguments(&commands).unwrap();

        assert_eq!(
            args,
            SetArgs {
                existence: Some(KeyExistence::XX),
                expiry: None,
                get: true
            }
        );
    }

    #[test]
    fn test_parse_ex() {
        let commands: Vec<String> = vec![String::from("EX"), String::from("100")];

        let args = parse_set_arguments(&commands).unwrap();

        assert_eq!(args.expiry, Some(KeyExpiry::EX(100)));
    }

    #[test]
    fn test_parse_ex_wrong_value() {
        let commands: Vec<String> = vec![String::from("EX"), String::from("value")];

        assert!(matches!(
            parse_set_arguments(&commands),
            Err(StorageError::CommandSyntaxError(_))
        ));
    }

    #[test]
    fn test_parse_ex_end_of_vector() {
        let commands: Vec<String> = vec![String::from("EX")];

        assert!(matches!(
            parse_set_arguments(&commands),
            Err(StorageError::CommandSyntaxError(_))
        ));
    }

    #[test]
    fn test_parse_px() {
        let commands: Vec<String> = vec![String::from("PX"), String::from("100")];

        let args = parse_set_arguments(&commands).unwrap();

        assert_eq!(args.expiry, Some(KeyExpiry::PX(100)));
    }

    #[test]
    fn test_parse_px_wrong_value() {
        let commands: Vec<String> = vec![String::from("PX"), String::from("value")];

        assert!(matches!(
            parse_set_arguments(&commands),
            Err(StorageError::CommandSyntaxError(_))
        ));
    }

    #[test]
    fn test_parse_px_end_of_vector() {
        let commands: Vec<String> = vec![String::from("PX")];

        assert!(matches!(
            parse_set_arguments(&commands),
            Err(StorageError::CommandSyntaxError(_))
        ));
    }

    #[test]
    fn test_parse_ex_and_px() {
        let commands: Vec<String> = vec![
            String::from("EX"),
            String::from("100"),
            String::from("PX"),
            String::from("100"),
        ];

        assert!(matches!(
            parse_set_arguments(&commands),
            Err(StorageError::CommandSyntaxError(_))
        ));
    }
}
