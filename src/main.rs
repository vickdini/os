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

//struct Char* buffer = (struct Char*) 0xb8000;

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

fn print_set_color(foreground: PrintColor, background: PrintColor)
{
    unsafe
    {
        COLOR = (foreground as u8) + ((background as u8) << 4);
    }
}

fn main() {
    //println!("Hello, world!");
    print_clear();
    print_set_color(PrintColor::Yellow, PrintColor::Black);
}