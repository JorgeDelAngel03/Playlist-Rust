use std::{
    cell::RefCell, cmp::Ordering, fs::File, fs::OpenOptions, io, io::prelude::*, io::BufRead,
    io::BufReader, io::Write, process::Command, rc::Rc,
}; // Módulos o crates (librerías)

fn clear_screen() { //Función para limpiar pantalla de la terminal en Windows
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", "cls"]).status().unwrap();
    } else {
        Command::new("clear").status().unwrap();
    }
}

fn wait_for_enter() { //Función para esperar la tecla enter del usuario
    println!("\nPresiona Enter para continuar...");
    let stdin = io::stdin();
    let _ = stdin.lock().lines().next(); //Bloqueamos la entrada mientras no se presione enter
}

#[derive(Clone, Debug)]  //Comportamiento por defecto y clon de la estructura Song
struct Song { //Estructura Song para las canciones
    name: String,
    author: String,
    genre: String,
    year: u16,
    language: String,
    url: String,
    genre_index: usize,
    language_index: usize,
    similarity: f32,
}

impl Song {} //No ocupé implementación xd

#[derive(Debug)] //Comportamiento por defecto de la estructura Playlist
struct Playlist { //Estructura Playlist para almacenar playlist
    name: String,
    songs: Vec<Song>,
}

impl Playlist { //Implementación de la estructura playlist
    fn new() -> Self { //Creación de una instancia de la estructura playlist
        Playlist {
            name: String::new(),
            songs: Vec::new(),
        }
    }

    fn add_song(&mut self, song: Song) { //Función para añadir canciones
        self.songs.push(song);
    }
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>; //Dato que nos sirve para establecer un vínculo entre la memoria utilizada por los datos del queue

struct Node<T> { //Estructura Nodo
    value: T,
    next: Link<T>,
}

pub struct Queue<T> { // Estructura Queue con head y tail
    head: Link<T>,
    tail: Link<T>,
}

impl<T> Queue<T> { //Implementación de la estructura Queue
    pub fn new() -> Self { //Creación de instancia de una cola
        Queue {
            head: None,
            tail: None,
        }
    }

    pub fn enqueue(&mut self, value: T) { //Método para encolar una canción
        let new_tail = Rc::new(RefCell::new(Node { value, next: None }));
        if let Some(old_tail) = self.tail.take() {
            old_tail.borrow_mut().next = Some(Rc::clone(&new_tail)); //Función para pedir prestado el siguiente nodo
        } else {
            self.head = Some(Rc::clone(&new_tail));
        }
        self.tail = Some(new_tail);
    }

    pub fn dequeue(&mut self) -> Option<T> { //Función para desencolar
        self.head.take().map(|old_head| {
            if let Some(next) = old_head.borrow_mut().next.take() {
                self.head = Some(next); //Remplazar el head si hay siguiente
            } else {
                self.tail.take();
            }
            Rc::try_unwrap(old_head).ok().unwrap().into_inner().value //Tratamos de hacerle un unwrapeo al head pa ver si tiene algo
        })
    }

    pub fn iter(&self) -> QueueIter<T> { //Aquí convertimos la cola a un iterador que nos permitirá hacer for eachs xd
        QueueIter {
            next: self.head.clone(),
        }
    }

    pub fn is_empty(&self) -> bool { //Comprobamos si el head no tiene nada
        self.head.is_none()
    }
}

pub struct QueueIter<T> { //Estructura del Iterador de la cola
    next: Link<T>,
}

impl<T> Iterator for QueueIter<T> //Implementación del Iterador de la cola
where
    T: Clone, //Clon del dato
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> { //El siguiente será igual al siguiente item xd (solo si hay)
        self.next.take().map(|current| {
            if let Some(next) = &current.borrow().next { 
                self.next = Some(Rc::clone(next));
            }
            current.borrow().value.clone()
        })
    }
}

fn similarity(genre_index: usize, language_index: usize, year: u16) -> Vec<Song> { //Aquí calculamos la similitud de la entrada con las canciones
    let songs: Vec<Song> = read().ok().unwrap(); //Hacemos un unwrap para conocer todo lo que alberga songs
    // Géneros que al final no ocupé definir
    /*let genre_names = vec![
        "Pop", "Kpop", "Jpop", "Rock", "Metal",
        "Reggaetón", "Rap", "Clásica", "Indie", "Jazz", "Electrónica"
    ];*/

    //Esta matriz de similitud compara todos los géneros entre sí, es subjetiva, pero este programa está hecho con motivos educativos :D
    let genre_similarity: [[f32; 11]; 11] = [
        [1.0, 0.7, 0.6, 0.5, 0.4, 0.4, 0.5, 0.3, 0.6, 0.4, 0.5], //Pop
        [0.7, 1.0, 0.8, 0.4, 0.3, 0.3, 0.4, 0.2, 0.5, 0.3, 0.4], //Kpop
        [0.6, 0.8, 1.0, 0.4, 0.3, 0.3, 0.4, 0.2, 0.5, 0.3, 0.4], //Jpop
        [0.5, 0.4, 0.4, 1.0, 0.7, 0.2, 0.5, 0.1, 0.4, 0.7, 0.3], //Rock
        [0.4, 0.3, 0.3, 0.7, 1.0, 0.1, 0.4, 0.1, 0.3, 0.6, 0.2], //Metal
        [0.4, 0.3, 0.3, 0.2, 0.1, 1.0, 0.7, 0.1, 0.2, 0.2, 0.3], //Reggaetón
        [0.5, 0.4, 0.4, 0.5, 0.4, 0.7, 1.0, 0.2, 0.3, 0.3, 0.4], //Rap
        [0.3, 0.2, 0.2, 0.1, 0.1, 0.1, 0.2, 1.0, 0.2, 0.2, 0.1], //Clásica
        [0.6, 0.5, 0.5, 0.4, 0.3, 0.2, 0.3, 0.2, 1.0, 0.5, 0.4], //Indie
        [0.4, 0.3, 0.3, 0.7, 0.6, 0.2, 0.3, 0.2, 0.5, 1.0, 0.3], //Jazz
        [0.5, 0.4, 0.4, 0.3, 0.2, 0.3, 0.4, 0.1, 0.4, 0.3, 1.0], //Electrónica
    ];

    //Esta matriz de similitud compara todos los lenguajes entre sí, es subjetiva, pero este programa está hecho con motivos educativos :D
    let language_similarity: [[f32; 8]; 8] = [
        [1.00, 0.30, 0.10, 0.10, 0.60, 0.80, 0.40, 0.50], // Español
        [0.30, 1.00, 0.10, 0.10, 0.40, 0.30, 0.60, 0.50], // Inglés
        [0.10, 0.10, 1.00, 0.50, 0.10, 0.10, 0.10, 0.50], // Coreano
        [0.10, 0.10, 0.50, 1.00, 0.10, 0.10, 0.10, 0.50], // Japonés
        [0.60, 0.40, 0.10, 0.10, 1.00, 0.70, 0.50, 0.50], // Francés
        [0.80, 0.30, 0.10, 0.10, 0.70, 1.00, 0.40, 0.50], // Italiano
        [0.40, 0.60, 0.10, 0.10, 0.50, 0.40, 1.00, 0.50], // Alemán
        [0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 1.00], // Sin letra
    ];
    let genre_weight = 0.6; //Asignamos un peso de 60% al género
    let language_weight = 0.2; //20% al idioma
    let year_weight = 0.2;  //20% también al año
    let max_difference = 2024 - 1950; //Esta es la máxima diferencia a considerar
    let mut vec_queue: Vec<Song> = Vec::new(); //Creamos un Vector de canciones que simula una cola o lista
    for mut song in songs { //Iteramos por cada song en songs
        let year_similarity: f64; //Declaramos el tipo de la similitud del año
        if year != 0 { 
            let difference = (year as i16 - song.year as i16).abs(); //Diferencia en i16 porque con otro tipo de dato no me dejaba xd
            year_similarity = 1.0 - (difference as f64 / max_difference as f64); //Similitud en f64
            year_similarity.max(0.0); //Nos aseguramos de que el valor no sea negativo
        } else { //Si selecciona música clásica, es más difícil determinar la similitud, así que le puse del 100% XD
            year_similarity = 1.0;
        }

        let total_similarity = 100 as f32
            * (genre_weight * genre_similarity[genre_index][song.genre_index]
                + language_weight * language_similarity[language_index][song.language_index]
                + year_weight as f32 * year_similarity as f32); //Sumamos todas las similitudes multiplicados por sus respectivos pesos, creo que así más o menos le hacen en machine learning, pero actualizando los pesos xd
        song.similarity = total_similarity; //Asignamos la similitud a la canción
        if total_similarity >= 75.00 { //Si tiene más del 75%, podemos decir que es una buena recomendación
            vec_queue.push(song); //Pusheamos la canción buena en el vector previamente creado
        }
    }
    //Ordemos las canciones de mayor a menos similitud
    vec_queue.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(Ordering::Equal)
    });
    vec_queue //Devolvemos este vector de canciones
}

fn read() -> std::io::Result<Vec<Song>> { //Aquí leemos el archivo de las canciones
    let file = File::open("canciones.txt")?; //Lo abrimos
    let buf_reader = BufReader::new(file); //Este buffer nos sirve para que no se nos petatee el programa

    let mut songs: Vec<Song> = Vec::new(); //Creamos un vector de canciones llamado songs
    for line in buf_reader.lines() { //Iteramos sobre cada línea del buffer
        let mut song = Song { //Instanciamos una canción con valores por defecto
            name: String::new(),
            author: String::new(),
            genre: String::new(),
            year: 0,
            language: String::new(),
            url: String::new(),
            genre_index: 0,
            language_index: 0,
            similarity: 100.00,
        };
        for (index, s) in line.unwrap().split("; ").enumerate() { //Separamos la entrada por el caracter ; 
            match index { //Aquí asignamos los valores a cada campo de la canción de acuerdo con el índice del iterador
                0 => song.name = s.to_string(),
                1 => song.author = s.to_string(),
                2 => song.genre = s.to_string(),
                3 => song.year = s.to_string().parse::<u16>().unwrap(), //Si no lo transformo a u16, se fresea
                4 => song.language = s.to_string(),
                5 => song.url = s.to_string(),
                _ => {} //Imposible que llegue aquí, a menos que se manipule indebidamente el archivo D:
            }
        }
        match song.genre.as_str() { //Aquí comparamos el género de la canción porque me encanta gastar memoria, los index solamente son para la tabla de similitud
            "Pop" => song.genre_index = 0,
            "Kpop" => song.genre_index = 1,
            "Jpop" => song.genre_index = 2,
            "Rock" => song.genre_index = 3,
            "Metal" => song.genre_index = 4,
            "Reggaetón" => song.genre_index = 5,
            "Rap" => song.genre_index = 6,
            "Clásica" => song.genre_index = 7,
            "Indie" => song.genre_index = 8,
            "Jazz" => song.genre_index = 9,
            "Electrónica" => song.genre_index = 10,
            _ => {}
        }
        match song.language.as_str() { //Lo mismo pero para el lenguaje, los index son para la tabla de similitud
            "Español" => song.language_index = 0,
            "Inglés" => song.language_index = 1,
            "Coreano" => song.language_index = 2,
            "Japonés" => song.language_index = 3,
            "Francés" => song.language_index = 4,
            "Italiano" => song.language_index = 5,
            "Alemán" => song.language_index = 6,
            "Sin letra" => song.language_index = 7,
            _ => {}
        }
        songs.push(song); //Pusheamos la canción ya después de modificarla en el vector de canciones
    }
    Ok(songs) //Regresamos las canciones en un vector
}

fn read_playlist() -> io::Result<Vec<Playlist>> { //Aquí leemos el archivo de las playlist
    let file = File::open("playlist.txt")?; //Abrimos el archivo de las playlist
    let buf_reader = BufReader::new(file); //Otro buffer muy importante
    let mut all_playlists: Vec<Playlist> = Vec::new(); //Creamos un mutable para que almacene todas las playlist
    let mut current_playlist: Option<Playlist> = None; //Esta es una playlist opcional, que nos dirá si existe en el archivo

    for line in buf_reader.lines() { //Leemos cada línea del buffer
        let line = line?; //Otra manera de hacerle el unwrap al buffer para que nos devuelva los valores como String
        if line.trim().starts_with('{') { //Si la línea comienza con {, creamos una nueva playlist
            if let Some(playlist) = current_playlist.take() {
                all_playlists.push(playlist); //Pusheamos la nueva playlist en el vector de playlist
            }
            let playlist_name = line.trim().trim_start_matches('{').trim().to_string(); //Aquí le añadimos el nombre a la playlist, que será lo que hay después del {
            current_playlist = Some(Playlist { //Instanciamos la playlist con su respectivo nombre
                name: playlist_name,
                songs: Vec::new(),
            });
        } else if line.trim().starts_with('}') { //Si la línea comienza con }, ya le bailó, aquí termina la playlist
            if let Some(playlist) = current_playlist.take() {
                all_playlists.push(playlist); //Pusheamos la playlist en el vector playlist
            }
        } else {
            if let Some(playlist) = &mut current_playlist { //Asignamos la playlist actual a esta playlist
                let mut song = Song { //Instanciamos una canción con valores por defecto
                    name: String::new(),
                    author: String::new(),
                    genre: String::new(),
                    year: 0,
                    language: String::new(),
                    url: String::new(),
                    genre_index: 0,
                    language_index: 0,
                    similarity: 100.00,
                };
                for (index, s) in line.split("; ").enumerate() { //Los volvemos a separar por ;
                    match index { //Asignamos todos los valores de acuerdo con el index
                        0 => song.name = s.to_string(),
                        1 => song.author = s.to_string(),
                        2 => song.genre = s.to_string(),
                        3 => song.year = s.to_string().parse::<u16>().unwrap(), //Igual acá, si no lo hago u16, se fresea xd
                        4 => song.language = s.to_string(),
                        5 => song.url = s.to_string(),
                        _ => {}
                    }
                }
                playlist.songs.push(song); //Pusheamos la canción dentro de la playlist actual
            }
        }
    }

    if let Some(playlist) = current_playlist { 
        all_playlists.push(playlist); //Pusheamos la playlist entera al vector de playlist
    }

    Ok(all_playlists) //Devolvemos el vector de playlist
}

fn write_in(content: String) -> std::io::Result<()> { //Aquí escribimos cosas en el archivo de playlist
    let mut file = OpenOptions::new()
        .append(true) //Aquí especificamos qué es lo que queremos abrir xd
        .create(true) //Si el archivo no existe, se crea, pero simón, sí existe
        .open("playlist.txt")?; //Abrimos el archivo de las playlist
    file.write_all(content.as_bytes())?; //Escribimos todo lo que hay en la variable contenido, que nos pasará otra función
    Ok(()) //Todo piola
}

fn print_playlists() -> io::Result<()> { //Realmente esta función no es necesaria porque después aprendí a hacerlo sin necesidad de hacer esta función
    let playlists = read_playlist()?; //Le hacemos un unwrap épico con el ? al vector de las playlist
    for playlist in playlists { //Iteramos por cada playlist en playlists, se escucha raro xd
        println!("\nPlaylist: {}", playlist.name); //Imprimimos el nombre de la playlist
        for song in playlist.songs { //Un for anidado :D, ahora iteramos por cada canción en el vector de playlist
            println!( //Imprimimos la canción actual con esos datos
                "{} - Autor: {} - Género: {} - Año: {} - Idioma: {} ",
                song.name, song.author, song.genre, song.year, song.language
            ) //Me acabo de enterar de que esto puede funcionar sin ;
        }
    }
    Ok(()) //Todo bien, todo correcto
}

fn main() { //Me da mucha flojera comentar el main
    let mut queue = Queue::new(); //Instanciamos un nuevo Queue
    loop { //Este loop carrea todo el programa
        clear_screen(); //Este es el único clear_screen que voy a comentar, limpia la pantalla de la terminal, aunque en el visual se fresea
        print!("¡Bienvenido al recomendador de canciones :D!\n¿En qué puedo ayudarte?");
        print!("\n\n1. Mostrar todas las canciones\n2. Recomendar playlist\n3. Reproducir playlist\n4. Mostrar todas las playlist\n5. Salir\n\nElige una opción: \n\n");
        let mut input = String::new(); //Instanciamos una entrada del usuario mutable
        let stdin = io::stdin();
        stdin
            .read_line(&mut input)
            .ok()
            .expect("No se puede leer esta línea");
        if input.trim() == "1" { //Aquí imprimimos todas las canciones
            clear_screen();
            let songs: Vec<Song> = read().ok().unwrap(); //Llamamos a la función que lee todas las canciones en el archivo
            for song in &songs { //Iteramos sobre cada canción en el vector de las canciones
                println!(
                    "{} - Autor: {} - Género: {} - Año: {} - Idioma: {}",
                    song.name, song.author, song.genre, song.year, song.language
                ); //Imprimimos toda la info, menos el URL porque esa cosa está gigantísima, de por sí no me esforcé para nada en el front 
            }
            wait_for_enter();
        } else if input.trim() == "2" { //Esta función está rotísima, aquí te recomienda las canciones, te guarda las playlist, tiene validaciones, etc
            for song in queue.iter() { //Mmm, realmente creo que este bloque ya no es necesario
                queue.dequeue();
            }
            loop { //Esto se repetirá mientras no selecciones un género válido
                clear_screen();
                println!("Veo que quieres que te recomiende canciones...");
                println!("¿Qué género quieres que te recomiende?");
                println!("\n1. Pop\n2. Kpop\n3. Jpop\n4. Rock\n5. Metal\n6. Reggaetón\n7. Rap\n8. Clásica\n9. Indie\n10. Jazz\n11. Electrónica");
                println!("\nElige una opción: \n");
                let mut input = String::new();
                let stdin = io::stdin();
                stdin
                    .read_line(&mut input)
                    .ok()
                    .expect("Failed to read line");
                let genre_index = input.trim().parse::<usize>().unwrap(); //Aquí se determina el index
                if genre_index >= 1 || genre_index <= 11 { //Acá si elegiste uno válido
                    loop { //Este se repite mientras no selecciones un idioma válido
                        clear_screen();
                        println!("¿Qué lenguaje prefieres escuchar?\n");
                        println!("1. Español\n2. Inglés\n3. Coreano\n4. Japonés\n5. Francés\n6. Italiano\n7. Alemán\n8. Sin letra");
                        println!("\nElige una opción: \n");
                        let mut input = String::new();
                        let stdin = io::stdin();
                        stdin
                            .read_line(&mut input)
                            .ok()
                            .expect("No se puede leer esta línea");
                        let language_index = input.trim().parse::<usize>().unwrap(); //Aquí mero se determina el index
                        if language_index >= 1 && language_index <= 8 { //Y acá se valida
                            loop {
                                if genre_index == 8 { //Si le gusta la música clásica, despreciamos el peso del año, realmente esto puede ser pulido de otra manera
                                    for song in similarity(genre_index - 1, language_index - 1, 0) {
                                        queue.enqueue(song);
                                    }
                                    for song in queue.iter() { //Acá se imprimen todas las canciones recomendadas al iterar sobre la cola
                                        println!(
                                            "Nombre: {} - Género: {} - Año: {} - Similitud Total: {:.2}",
                                            song.name, song.genre, song.year, song.similarity
                                        );
                                    }
                                    break;
                                } else { //Si no escogió clásica, entonces realizamos otro cálculo
                                    clear_screen();
                                    println!("¿Qué época prefieres escuchar?");
                                    println!("\nIngresa un año desde 1950 hasta 2024: \n");
                                    let mut input = String::new();
                                    let stdin = io::stdin();
                                    stdin
                                        .read_line(&mut input)
                                        .ok()
                                        .expect("No se puede leer esta línea");
                                    let year = input.trim().parse::<u16>().unwrap(); //Convertimos el años a u16
                                    if year >= 1950 && year <= 2024 { //Aquí verificamos que haya ingresado un año válido
                                        clear_screen();
                                        for song in
                                            similarity(genre_index - 1, language_index - 1, year)
                                        {
                                            queue.enqueue(song);
                                        } //En este for anterior, se itera sobre cada canción que exista en el vector que nos devuelve la función de similitud, y se encola en la cola jaja
                                        if !queue.head.is_none() { //Si la cola no está vacía, esto va a charchar
                                            print!("Playlist:\n");
                                            for song in queue.iter() { //Iteramos sobre cada canción de la cola y la imprimimos
                                                println!(
                                                    "{} - Género: {} - Año: {} - Idioma: {} - Similitud: {:.2}",
                                                    song.name,
                                                    song.genre,
                                                    song.year,
                                                    song.language,
                                                    song.similarity
                                                );
                                            }
                                            loop { //Aquí te permitirá guardar o no la playlist que te recomendó
                                                println!("\n¿Quieres guardar la playlist?");
                                                println!("\nIngresa 'GUARDAR' para guardar la playlist\nO cualquier otra cosa para continuar\n\n");
                                                let mut input = String::new();
                                                let stdin = io::stdin();
                                                stdin
                                                    .read_line(&mut input)
                                                    .ok()
                                                    .expect("No se puede leer esta línea");
                                                if input.trim() == "GUARDAR" { //Si es exactamente GUARDAR, gritando, realiza lo siguiente
                                                    if !queue.head.is_none() { //Si no está vacía, te permite guardarla
                                                        loop {
                                                            clear_screen();
                                                            println!("¿Qué nombre quieres darle a la playlist?");
                                                            println!(
                                                            "\nIngresa el nombre de la playlist: "
                                                        );
                                                            let mut input = String::new();
                                                            let stdin = io::stdin();
                                                            stdin
                                                                .read_line(&mut input)
                                                                .ok()
                                                                .expect(
                                                                    "No se puede leer esta línea",
                                                                );
                                                            if input.contains('{')  //Si el nombre contiene {}, no se puede guardar, porque esta es la manera en la que se leen las playlist
                                                                || input.contains('}')
                                                            {
                                                                println!("El nombre de la playlist no puede contener los caracteres {{ }}");
                                                                wait_for_enter();
                                                            } else {
                                                                let mut content = "".to_string();
                                                                let playlist_name = input.trim();
                                                                let playlists =
                                                                    read_playlist().unwrap();
                                                                let mut exist = false;
                                                                for playlist in playlists { //Iteramos sobre cada playlist en el vector para verificar si ya existe una playlist con ese nombre
                                                                    if playlist.name
                                                                        == playlist_name
                                                                    {
                                                                        println!("Ya existe la playlist {} D:!", playlist_name);
                                                                        wait_for_enter();
                                                                        exist = true;
                                                                        break;
                                                                    }
                                                                }
                                                                if !exist { //Si no existía, se crea una nueva playlist, estableciendo el formato correcto para leerla posteriormente
                                                                    let separator = "; ";
                                                                    content.push_str("{");
                                                                    content.push_str(playlist_name);
                                                                    for song in queue.iter() {
                                                                        content.push_str(&format!(
                                                                            "\n{}",
                                                                            song.name
                                                                        ));
                                                                        content.push_str(&format!(
                                                                            "{}{}",
                                                                            separator, song.author
                                                                        ));
                                                                        content.push_str(&format!(
                                                                            "{}{}",
                                                                            separator, song.genre
                                                                        ));
                                                                        content.push_str(&format!(
                                                                            "{}{}",
                                                                            separator, song.year
                                                                        ));
                                                                        content.push_str(&format!(
                                                                            "{}{}",
                                                                            separator,
                                                                            song.language
                                                                        ));
                                                                        content.push_str(&format!(
                                                                            "{}{}",
                                                                            separator, song.url
                                                                        ));
                                                                    }
                                                                    content.push_str("\n}\n");
                                                                    write_in(content); //Aquí llamamos a la función para agregar la playlist al arhivo
                                                                    break;
                                                                }
                                                            }
                                                        }
                                                        break;
                                                    } else {
                                                        println!(
                                                        "No se puede guardar una playlist vacía D:"
                                                    );
                                                        wait_for_enter();
                                                        break;
                                                    }
                                                } else {
                                                    println!("Está bien, la playlist actual será eliminada...");
                                                    for son in queue.iter() { //Si no se guardo, se itera sobre cada canción en la playlist y se desencola
                                                        queue.dequeue();
                                                    }
                                                    wait_for_enter();
                                                    break;
                                                }
                                            }

                                            break;
                                        } else { //Si de plano metes algo raro en los menús, no te va a recomendar nada
                                            println!("\nNo se encontraron coincidencias D:");
                                            wait_for_enter();
                                            break;
                                        }
                                    } else {
                                        println!("Opción inválida D:!");
                                        wait_for_enter();
                                    }
                                }
                            }
                            break;
                        } else {
                            println!("Opción inválida D:!");
                            wait_for_enter();
                        }
                    }
                    break;
                } else {
                    println!("Opción inválida D:!");
                    wait_for_enter();
                }
            }
        } else if input.trim() == "3" { //Aquí "Reproducimos" la playlist, aunque en verdad solo pongo los links a youtube XD
            clear_screen();
            if !queue.head.is_none() { //Si no está vacía, "Reproducimos" la playlist actual
                for song in queue.iter() { //Reproducimos cada canción hasta que el usuario de un enter
                    clear_screen();
                    println!("Reproduciendo playlist...");
                    println!("Canción actual: {}, {}", song.name, song.author);
                    println!("URL: {}\n", song.url);
                    wait_for_enter();
                }
            } else { //Si está vacía, permitimos al usuario que ingrese el nombre de alguna playlist
                println!("Ingresa el nombre de la Playlist que quieres reproducir:\n");
                let mut input = String::new();
                let stdin = io::stdin();
                stdin
                    .read_line(&mut input)
                    .ok()
                    .expect("No se puede leer esta línea");
                let playlist_name = input.trim();
                let playlists = read_playlist().unwrap();
                let mut exist = false;
                for playlist in playlists { //Acá verificamos si existe la playlist, si no, pues no reproduce nada
                    if playlist.name == playlist_name {
                        exist = true;
                        for song in playlist.songs { //Acá se encola cada canción de la playlist encontrada
                            queue.enqueue(song);
                        }
                        break;
                    }
                }
                if exist { //Si existe la playlist, se itera sobre cada canción de esta, para luego reproducirla
                    for song in queue.iter() {
                        clear_screen();
                        println!("Reproduciendo playlist...");
                        println!("Canción actual: {}, {}", song.name, song.author);
                        println!("URL: {}\n", song.url);
                        wait_for_enter();
                    }
                } else { //Si no se encontró, ni cómo hacerle, de vuelta pal menú
                    println!("No se encontró la playlist {} D:", playlist_name);
                    wait_for_enter();
                }
            }
            for song in queue.iter() { //Acá desencolamos todas las canciones
                queue.dequeue();
            }
        } else if input.trim() == "4" { //Acá mandamos a llamar a la función para imprimir todas las playlist
            clear_screen();
            print_playlists(); //Llamada a la función para imprimir las playlist
            wait_for_enter();
        } else if input.trim() == "5" { //Acá terminamos el programa
            clear_screen();
            println!("Gracias por utilizar el programa :D!");
            wait_for_enter();
            break;
        } else { //Esto en dado caso de que el usuario ingrese algo que no debe D:
            println!("Opción inválida D:!");
            wait_for_enter();
        }
    }
}