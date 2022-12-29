<h1 align="center">elem</h1>

<p align="center">
  <b>Logitech Battery Level Tray Indicator</b>
</p>

Show your wireless Logitech devices battery levels in your system tray.

The name comes from the Hungarian word for battery: elem.

## Usage

### Installing from [crates.io](https://crates.io)

To install from [crates.io](https://crates.io) you must have
[Rust](https://www.rust-lang.org/) installed.

```shell
cargo install elem --force
```

### Downloading a Prebuilt Binary

To start using elem, download and execute the latest executable from the
releases page:
[Fuwn/elem/releases/latest](https://github.com/Fuwn/elem/releases/latest).

### Building Yourself

To build elem, you must have [Rust](https://www.rust-lang.org/) and
[Git](https://git-scm.com/) installed on your system.

1. Clone this repository: `git clone https://github.com/Fuwn/elem.git`
2. Navigate into your local repository: `cd elem`
3. Build elem: `cargo build elem --release`
4. elems executable will be located at `target/release/elem`

## Notes

### Update Frequency

By default, elem fetches the selected devices battery level every minute. This
should be more than enough for most people considering how well Logitech devices
conserve power.

If you would like to increase -- or decrease -- the update frequency, you can
launch elem from the command-line and pass a value in milliseconds which will be
your new update frequency.

```shell
$ ./elem 60000  # Updates every 60 seconds (60000ms / 1000ms = 60s)
$ ./elem 1000   # Updates every second (1000ms = 1s)
$ ./elem 120000 # Updates every two minutes (120000ms / 1000ms = 120s)
```

### Frozen?

If elem seems frozen, it isn't. It's just waiting for watchman (battery level
fetcher) to return a value.

### Solution

Writing this project was actually pretty interesting.

The main problem I had to solve was how I was displaying the battery level from
just the number. My first thought was just downloading a bunch of icons and
bundling them together with the program. Soon I realized that it's actually not
that easy to find royalty free icons that don't look terrible. On top of that,
do you really want to carry around 100+ icons with you just to display a number?
I ditched that  and started going down the procedural route. I tried to locate a
Rust crate that generated images from text, but surprisingly, I couldn't find
any. Just numbers to text? Still, nope. This lead me to my next item to solve:
generating images from numbers.

My first idea which I ended up using is generating ASCII art from numbers.
Once again, to my surprise, I couldn't find a crate that did this. (Side note;
I actually ended up finding a crate that somewhat accomplished this, but by the
time I did, I had already finished my own implementation. I actually prefer my
implementation to the crate as this gives me a lot more control over the final
result.) Using
[Patrick Gillespie's Text to ASCII Art Generator (TAAG)](http://www.patorjk.com/software/taag/#p=display&f=ANSI%20Regular&t=Type%20Something%20),
I was able to extract a few important ASCII art pieces: numbers zero through
nine; and eventually, I ended up needing a set of ellipsis, a question mark, and
a smiley face.

Using these fundamental pieces, I was able to write a set of functions to
generate ASCII art any number combination I wanted. The great thing about this
is that it's all ASCII characters, meaning that I can easily iterate over the
lines and characters and convert it into a ad-hoc pixel art. In just a few
simple steps, I could take any ASCII pixel art and convert it into a RGBA image
using these steps:

1. Find the width of the ASCII art
2. Strip the newline characters so that the entire ASCII art is on one line
3. Iterate over each character in the ASCII art, writing the corresponding
   pixel value to an image buffer: 0 0 0 0 for an empty pixel and 255 255 255
   255 for a filled pixel (red, green, blue, alpha).
4. Save the image buffer to memory and use it as the icon for the tray
   indicator. :)

Pretty cool, right?

In the future, I'll see if I can optimize this process a bit more, but since the
main bottleneck is Logitech G HUBs API, it's not that big of a deal.

## License

This project is licensed with the [GNU General Public License v3.0](LICENSE).
