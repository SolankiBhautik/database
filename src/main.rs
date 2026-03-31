use std::{fs::{OpenOptions}, io::{Read}};

#[derive(Debug)]
struct User {
    id: u32,
    name: String
}


impl User {
    fn to_byts(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let id_bytes = self.id.to_le_bytes();
        let name_bytes = self.name.to_string().into_bytes();

        id_bytes.into_iter().for_each(|b| bytes.push(b));
        name_bytes.into_iter().for_each(|b| bytes.push(b));

        bytes
    }


    fn from_byts(bytes: &[u8]) -> Self {
        let id_bytes: [u8; 4] = bytes[0..4].try_into().unwrap();
        let id = u32::from_le_bytes(id_bytes);

        let name_bytes: Vec<u8> = bytes[4..bytes.len()].try_into().unwrap();
        let name = String::from_utf8(name_bytes).unwrap();

        User {
            id,
            name
        }
    }
}

fn main() {
    let user = User {
        id: 1,
        name: "bhautik".to_string()
    };

    let mut file = OpenOptions::new()
                                .write(true)
                                .create(true)
                                .read(true)
                                .open("db.bd")
                                .unwrap();

    // file.write_all(&user.to_byts()).unwrap();


    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).unwrap();

    let user = User::from_byts(&bytes);

    dbg!(user);
}
