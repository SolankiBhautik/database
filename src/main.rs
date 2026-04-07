use std::{fs::{File, OpenOptions}, io::{Read, Seek, Write}, os::unix::fs::MetadataExt};

#[derive(Debug)]
struct User {
    id: u32,
    name: String
}

impl User {
    fn to_byts(&self) -> Vec<u8> {
        let id_bytes = self.id.to_le_bytes();
        let name_len_bytes = (self.name.len() as u16).to_le_bytes();
        let name_bytes = self.name.to_string().into_bytes();
        [
            id_bytes.as_slice(),
            name_len_bytes.as_slice(),
            name_bytes.as_slice(),
        ].concat()
    }


    fn from_byts(bytes: &[u8]) -> Self {
        let id_bytes: [u8; 4] = bytes[0..4].try_into().unwrap();
        let id = u32::from_le_bytes(id_bytes);

        let name_len_bytes: [u8; 2] = bytes[4..6].try_into().unwrap();
        let name_len = u16::from_le_bytes(name_len_bytes);

        let name_bytes: Vec<u8> = bytes[6.. (6 + name_len) as usize].try_into().unwrap();
        let name = String::from_utf8(name_bytes).unwrap();

        User {
            id,
            name
        }
    }
}


#[derive(Debug)]
struct DB  {
    free_start: u16,
    free_end: u16,
    file: File
}

const PAGE_SIZE: u16 = 4096;

fn main() {
    let users = vec![
        User { id: 1, name: "bhautik".to_string() },
        User { id: 2, name: "badal".to_string() },
        User { id: 3, name: "user3".to_string() },
        User { id: 4, name: "user4".to_string() },
        User { id: 5, name: "user5".to_string() },
        User { id: 6, name: "user6".to_string() },
        User { id: 7, name: "user7".to_string() },
        User { id: 8, name: "user8".to_string() },
        User { id: 9, name: "user9".to_string() },
        User { id: 10, name: "user10".to_string() },
    ];

    let mut db = init_db("db.bd");

    for user in &users {
        insert(&mut db, user);
    }

    // let users1 = read_all(&mut db);

    // dbg!(users1);
    
    let user = find_by_id(&mut db, 10);
    dbg!(user);

}


fn find_by_id(db: &mut DB, id: u32) -> Option<User> {
    for i in (4..db.free_start).step_by(4) {
        db.file.seek(std::io::SeekFrom::Start(i as u64)).unwrap();
        let mut user_s = [0u8; 2];
        db.file.read_exact(&mut user_s).unwrap();
        let user_s = u16::from_le_bytes(user_s);

        db.file.seek(std::io::SeekFrom::Start((i + 2) as u64)).unwrap();
        let mut user_e = [0u8; 2];
        db.file.read_exact(&mut user_e).unwrap();
        let user_e = u16::from_le_bytes(user_e);

        db.file.seek(std::io::SeekFrom::Start(user_s as u64)).unwrap();
        let mut user = vec![0u8; (user_e - user_s) as usize];
        db.file.read_exact(&mut user).unwrap();
        let user = User::from_byts(&user);
        if user.id == id {
            return Option::Some(user);
        }
    }

    Option::None
}

fn insert(db: &mut DB, user: &User) {
    let user_bytes = user.to_byts();
    let user_start  = db.free_end - user_bytes.len() as u16;
    let user_end = db.free_end.clone();

    // insert slot 
    let free_start = user_start.to_le_bytes();
    let free_end = user_end.to_le_bytes();
    db.file.seek(std::io::SeekFrom::Start(db.free_start as u64)).unwrap();
    db.file.write_all(&[free_start, free_end].concat()).unwrap();

    // insert user
    db.file.seek(std::io::SeekFrom::Start(user_start as u64)).unwrap();
    db.file.write_all(&user_bytes).unwrap();

    let new_start = db.free_start + 4;
    let new_end = db.free_end - user_bytes.len() as u16;
    let free_start = new_start.to_le_bytes();
    let free_end = new_end.to_le_bytes();
    db.file.seek(std::io::SeekFrom::Start(0)).unwrap();
    db.file.write_all(&[free_start, free_end].concat()).unwrap();
    db.free_start = new_start;
    db.free_end = new_end;
}

fn read_all(db: &mut DB) -> Vec<User> {
    let mut users = Vec::new();

    for i in (4..db.free_start).step_by(4) {
        db.file.seek(std::io::SeekFrom::Start(i as u64)).unwrap();
        let mut user_s = [0u8; 2];
        db.file.read_exact(&mut user_s).unwrap();
        let user_s = u16::from_le_bytes(user_s);

        db.file.seek(std::io::SeekFrom::Start((i + 2) as u64)).unwrap();
        let mut user_e = [0u8; 2];
        db.file.read_exact(&mut user_e).unwrap();
        let user_e = u16::from_le_bytes(user_e);

        db.file.seek(std::io::SeekFrom::Start(user_s as u64)).unwrap();
        let mut user = vec![0u8; (user_e - user_s) as usize];
        db.file.read_exact(&mut user).unwrap();
        let user = User::from_byts(&user);
        users.push(user);
    }

    users
}

fn init_db(path: &str) -> DB {
    let mut file = OpenOptions::new()
                                .write(true)
                                .create(true)
                                .read(true)
                                .open(path)
                                .unwrap();

    let size = file.metadata().unwrap().size();

    if size == 0 {
        let free_start = (4 as u16).to_le_bytes();
        let free_end = (PAGE_SIZE as u16).to_le_bytes();
        file.write_all(&[free_start, free_end].concat()).unwrap();
        return DB {
            free_start: 4,
            free_end: PAGE_SIZE,
            file: file
        }
    }

    file.seek(std::io::SeekFrom::Start(0)).unwrap();
    let mut free_start = [0u8; 2];
    file.read_exact(&mut free_start).unwrap();
    let free_start = u16::from_le_bytes(free_start);

    file.seek(std::io::SeekFrom::Start(2)).unwrap();
    let mut free_end = [0u8; 2];
    file.read_exact(&mut free_end).unwrap();
    let free_end = u16::from_le_bytes(free_end);

    DB {
        free_start,
        free_end,
        file
    }
}



