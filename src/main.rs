static NUM_COLS: usize = 80;
static NUM_ROWS: usize = 25;

#[derive(Copy, Clone)]
struct Char
{
    character: u8,
    color: u8
}

enum PrintColor
{
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

static mut BUFFER: *mut Char = 0xb8000 as *mut Char;

static mut COL: usize = 0;
static mut ROW: usize = 0;
static mut COLOR: u8 = (PrintColor::White as u8) | (PrintColor::Black as u8) << 4;

unsafe fn clear_row(row_id: usize)
{
    let empty = Char
    {
        character: ' ' as u8,
        color: COLOR
    };
 

    for col_id in 0..NUM_COLS
    {
        *(BUFFER.wrapping_add(col_id + NUM_COLS * row_id)) = empty;
    }
}

fn print_clear()
{
    for i in 0..NUM_ROWS
    {
        unsafe
        {
            clear_row(i);
        }
    }
}

unsafe fn print_newline()
{
    COL = 0;

    if ROW < NUM_ROWS - 1
    {
        ROW += 1;
        return;
    }

    for row_id in 1..NUM_ROWS
    {
        for col_id in 0..NUM_COLS
        {
            let character = *(BUFFER.wrapping_add(col_id + NUM_COLS * row_id));
            *(BUFFER.wrapping_add(col_id + NUM_COLS * (row_id - 1))) = character;
        }
    }

    clear_row(NUM_ROWS - 1);
}

unsafe fn print_char(character: char)
{
    if character == '\n'
    {
        print_newline();
        return;
    }

    if COL > NUM_COLS
    {
        print_newline();
    }
 
    *(BUFFER.wrapping_add(COL + NUM_COLS * ROW)) = Char
    {
        character: character as u8,
        color: COLOR,
    };

    COL += 1;
}

fn print_str(str: &str)
{
    for character in str.chars()
    {
        unsafe{ print_char(character); }
    }
}

fn print_set_color(foreground: PrintColor, background: PrintColor)
{
    unsafe
    {
        COLOR = (foreground as u8) + ((background as u8) << 4);
    }
}

fn main() {
    print_clear();
    print_set_color(PrintColor::Yellow, PrintColor::Black);
    print_str("Welcome to Hydra in rust!\n");
}