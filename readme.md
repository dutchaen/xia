# xia
raw http request parser (supports Go and Python only)


## what is this lol?
this program will take raw http headers as input along with an optional body string and parse it into psuedo code

## why should i use this
well idk, if you are a reverse engineer or a webscraper, this will make your life alot easier when you copy http requests

also depending on the "Content-Type" header, it automatically parse the body as a map or pretty json string (i think it makes things easier to manage that way)

## well idk what ur talking ab, show me an example?
ok sure
### > so we need to copy the raw headers to your clipboard first (firefox lets me do this easy)
![get_raw_headers](https://i.ibb.co/k0JkfX4/image.png)
![copy_raw_header_text](https://i.ibb.co/sJT2QfL/image.png)

### next you need to run the program (im using cargo run but u can run xia directly)
![run_program](https://i.ibb.co/Rb0XzPc/image.png)

pls note: the program will detect if you are sending a GET request. if you are not sending a GET request, you will see this prompt.
if you dont want a response body, type '*' and press enter, otherwise copy the response text to your clipboard and press enter


### after the full http request is copied, you can select one of the following options and after that you should be good to go
![aftermath](https://i.ibb.co/MgrhLq0/image.png)

### what the result of this looks like...
![result](https://i.ibb.co/NFz7RqM/image.png)


## how do i use this?
this tool only works on windows for now bc of the clipboard_win crate (NO LINUX OR MAC OS SUPPORT NERDS)

aside from that, check the [release section](https://github.com/dutchaen/xia/releases/tag/release) of this repository if you dont have rust installed, there should be an prebuilt exe for you to use

if you want to build from src, download rust, git clone this repo, cd into it then run ```cargo build``` or ```cargo build --release```
