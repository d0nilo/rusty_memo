use std::{io::{self, stdout}, thread, time::{self, Duration}, char};
use rand::Rng;
use crossterm::{
    cursor::{EnableBlinking, MoveTo, RestorePosition, SavePosition}, 
    event::{self, KeyCode, KeyEvent}, 
    terminal::{EnterAlternateScreen, LeaveAlternateScreen}, 
    ExecutableCommand,
};

fn gen_game(field: &mut [[u32; 10]; 10], vector: &mut [u32; 50]) {
    let mut buff: u32;

    for i in 0..=10 - 1 {
        for j in 0..=10 - 1 {
            loop {
                buff = rand::thread_rng().gen_range(1..=50);
                if vector[buff as usize - 1] < 2{
                    vector[buff as usize - 1] += 1;
                    break;
                }
            }
            field[i][j] = buff;
        }
    }
}

fn update_matrix(matrix: &mut [[char; 31]; 21], field: &mut [[u32; 10]; 10],init: &mut bool, cur_x: u16, cur_y: u16) -> bool{
    if *init {
        matrix[0][0] = '╔';
        matrix[0][30] = '╗';
        matrix[20][0] = '╚';
        matrix[20][30] = '╝';
        for i in 1..=31 - 2 {
            matrix[0][i] = '═';
            matrix[20][i] = '═';
        }
        for i in (3..=31 - 2).step_by(3) {
            matrix[0][i] = '╦';
            matrix[20][i] = '╩';
        }

        for j in 1..=21 - 2 {
            for i in (3..=31).step_by(3) {
                if j % 2 == 0 {
                    matrix[j][i] = '╬';
                    matrix[j][i - 1] = '═';
                    matrix[j][i - 2] = '═';
                }
                else {
                    matrix[j][i] = '║';
                    matrix[j][i - 1] = ' ';
                    matrix[j][i - 2] = ' ';
                }
            }
        }

        for j in 1..=21 - 2 {
            if j % 2 == 0 {
                matrix[j][0] = '╠';
                matrix[j][30] = '╣';
            }
            else {
                matrix[j][0] = '║';
                matrix[j][30] = '║';
            }
        }

        *init = false;
        return false;
    }
    else {
        if matrix[cur_y as usize][cur_x as usize] != ' ' {
            return false;
        }
        else {
            let mut offset_x;
            let mut offset_y = 1;
            for i in 0..=10 - 1 {
                offset_x = 0;
                for j in 0..=10 - 1 {
                    if (i + offset_y == cur_y) && (j + offset_x + 2 == cur_x) {
                        if field[i as usize][j as usize] < 10 {
                            matrix[(i + offset_y) as usize][(j + 2 + offset_x) as usize] = char::from_digit(field[i as usize][j as usize], 10).unwrap();
                        }
                        else {
                            let num_str = field[i as usize][j as usize].to_string();
                            let chars: Vec<char> = num_str.chars().collect();
                            matrix[(i + offset_y) as usize][(j + 2 + offset_x - 1) as usize] = chars[0];
                            matrix[(i + offset_y) as usize][(j + 2 + offset_x) as usize] = chars[1];
                        }
                    }
                    offset_x += 2;
                }
                offset_y += 1;
            }
            return true;
        }
    }
}

fn update_screen(matrix: [[char; 31]; 21], count_guessed: u32) {
    for i in 0..=21 - 1 {
        for j in 0..=31 - 1 {
            print!("{}", matrix[i][j]);
        }
        print!("\n\r");
    }
    println!("Guessed: {}\n\rArrows to move\n\rSpace to select\n\rEsc to exit", count_guessed);
}

fn check_cells(matrix: &mut [[char; 31]; 21], mem_vec: [u16; 4], field: [[u32; 10]; 10], count_guessed: &mut u32) {
    let mut offset_x;
    let mut offset_y = 1;
    let mut number_found = false;
    let mut found_coords: [u16; 2] = [0; 2];
    for i in 0..=10 - 1 {
        offset_x = 0;
        for j in 0..=10 - 1 {
            if !number_found && (((i + offset_y == mem_vec[1]) && (j + offset_x + 2 == mem_vec[0])) || ((i + offset_y == mem_vec[3]) && (j + offset_x + 2 == mem_vec[2]))) {
                found_coords[0] = i;
                found_coords[1] = j;
                number_found = true;
            }
            else if number_found && (((i + offset_y == mem_vec[1]) && (j + offset_x + 2 == mem_vec[0])) || ((i + offset_y == mem_vec[3]) && (j + offset_x + 2 == mem_vec[2]))) {
                if field[i as usize][j as usize] == field[found_coords[0] as usize][found_coords[1] as usize] {
                    *count_guessed += 1;
                    return;
                }
                else {
                    matrix[mem_vec[1] as usize][mem_vec[0] as usize] = ' ';
                    matrix[mem_vec[1] as usize][mem_vec[0] as usize - 1] = ' ';
                    matrix[mem_vec[3] as usize][mem_vec[2] as usize] = ' ';
                    matrix[mem_vec[3] as usize][mem_vec[2] as usize - 1] = ' ';
                    return;
                }
            }
            offset_x += 2;
        }
        offset_y += 1;
    }
}

fn main() -> crossterm::Result<()> {
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;
    io::stdout().execute(EnableBlinking)?;

    let mut cur_x: u16 = 2;
    let mut cur_y: u16 = 1;
    let mut matrix = [[' '; 31]; 21];
    let mut field: [[u32; 10]; 10] = [[0; 10]; 10];
    let mut vector: [u32; 50] = [0; 50];
    let mut mem_vec: [u16; 4] = [0; 4]; //{1_cur_x; 1_cur_y; 2_cur_x; 2_cur_y}
    let mut count_opened = 0;
    let mut count_guessed = 0;
    let mut init = true;
    let mut win = false;
    let mut cheat = false;

    gen_game(&mut field, &mut vector);
    io::stdout().execute(MoveTo(1, 2))?;
    update_matrix(&mut matrix, &mut field, &mut init, cur_x, cur_y);
    io::stdout().execute(MoveTo(0, 0))?;
    update_screen(matrix, count_guessed);
    io::stdout().execute(MoveTo(cur_x, cur_y))?;

    loop {
        if event::poll(Duration::from_millis(500))? {
            if let event::Event::Key(KeyEvent {code, modifiers: _,}) = event::read()? {
                if !win {
                    match code {
                        KeyCode::Down => {
                            if cur_y < 19 {
                                cur_y += 2;
                                io::stdout().execute(MoveTo(cur_x, cur_y))?;
                            }
                        },
                        KeyCode::Up => {
                            if cur_y > 1 {
                                cur_y -= 2;
                                io::stdout().execute(MoveTo(cur_x, cur_y))?;
                            }
                        },
                        KeyCode::Left => {
                            if cur_x > 2 {
                                cur_x -= 3;
                                io::stdout().execute(MoveTo(cur_x, cur_y))?;
                            }
                        },
                        KeyCode::Right => {
                            if cur_x < 29 {
                                cur_x += 3;
                                io::stdout().execute(MoveTo(cur_x, cur_y))?;
                            }
                        },
                        KeyCode::Char(' ') => {
                            io::stdout().execute(SavePosition)?;
                            let state = update_matrix(&mut matrix, &mut field, &mut init, cur_x, cur_y);
                            io::stdout().execute(MoveTo(0, 0))?;
                            update_screen(matrix, count_guessed);
                            io::stdout().execute(RestorePosition)?;
                            if state {
                                count_opened += 1;
                                if count_opened == 2 {
                                    mem_vec[2] = cur_x;
                                    mem_vec[3] = cur_y;

                                    check_cells(&mut matrix, mem_vec, field, &mut count_guessed);
                                    thread::sleep(time::Duration::from_millis(500));
                                    mem_vec = [0; 4];
                                    io::stdout().execute(SavePosition)?;
                                    io::stdout().execute(MoveTo(0, 0))?;
                                    update_screen(matrix,count_guessed);
                                    io::stdout().execute(RestorePosition)?;

                                    if count_guessed == 50 {
                                        win = true;
                                        clearscreen::clear().expect("failed to clear screen");
                                        print!("\n\r╔                      ╗
                                                  \r║       You win!       ║
                                                  \r╚                      ╝");
                                        println!("\n\rEsc to exit");
                                    }

                                    count_opened = 0;
                                }
                                else {
                                    mem_vec[0] = cur_x;
                                    mem_vec[1] = cur_y;
                                }
                            }
                        }
                        KeyCode::F(1) => {
                            io::stdout().execute(MoveTo(40, 0))?;
                            if !cheat {
                                let pox = cur_x;
                                let poy = cur_y;
                                for i in 0..=10 - 1 {
                                    for j in 0..=10 - 1 {
                                        print!("{}\t", field[i][j]);
                                    }
                                    io::stdout().execute(MoveTo(40, i as u16 + 1))?;
                                }
                                io::stdout().execute(MoveTo(pox, poy))?;
                                cheat = true;
                            }
                            else {
                                let pox = cur_x;
                                let poy = cur_y;
                                clearscreen::clear().expect("failed to clear screen");
                                update_screen(matrix,count_guessed);
                                io::stdout().execute(MoveTo(pox, poy))?;
                                cheat = false;
                            }
                        }
                        KeyCode::Esc => {
                            break;
                        }
                        _ => {}
                    }
                }
                else {
                    match code {
                        KeyCode::Esc => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
        thread::sleep(time::Duration::from_millis(10));
    }
    stdout.execute(LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
