<div align="center">
<img src="logo.png" width="256" height="256" alt-text="health-assistant-icon">

**An AI assistant that monitors your health and provides suggestions for improvement.**

[Features](#features) •
[Getting Started](#getting-started) •
[Storage](#storage) •
[Usage](#usage)

</div>

# Features
- **Instant Messaging**: Engage in real-time conversations with friends and colleagues.
- **AI-Powered Responses**: Toggle AI responses on and off to enhance your messaging experience.
- **Diverse AI Models**: Choose from a wide selection of AI models tailored to your needs.
- **Live Status Updates**: Stay informed with live updates on your contacts' online, offline, or idle status.
- **Connect with Friends**: Send and receive friend requests to build your network.
- **Invite and Share**: Easily invite others to your conversations and share your experiences.
- **File Sharing**: Attach and share files up to 10 MB effortlessly.
- **Health Tracking**: Fill out health forms to monitor your habits and well-being over time.
- **Visualize Your Data**: Use graphs to visualize statistics from your health forms.
- **Export Data**: Export your form data for further analysis or record-keeping.
- **Advanced Search**: Utilize full-text searching, custom filters, and sorting options to quickly find specific messages.

# Getting Started
To build project you must have [cargo](https://www.rust-lang.org/tools/install) and [node](https://nodejs.org/en) installed on your system.
1. Clone the repo locally
```
git clone https://github.com/Aappo001/AI-Personal-Health-Assistant.git
cd AI-Personal-Health-Assistant
```
2. Set required environment variables inside a `.env` file. The JWT_KEY variable must be set in order for the project to compile. The value of the variable does not matter, just make sure it is consistent. HF_API_KEY is needed to generate AI responses, you can get yours [here](https://huggingface.co/settings/tokens)
```
cd api
echo "JWT_KEY={YOUR_JWT_KEY}" >> .env
echo "HF_API_KEY={YOUR_API_KEY}" >> .env
```
3. Build the backend with cargo
```
cargo b -r
```
4. Build the frontend with vite/npm
```
cd ../client
npm i
npm run build
```
5. Start up the backend server
```
cd ../api
cargo r -r
```
6. Visit `https://localhost:3000` in your browser

# Storage
All of the data of the application is stored inside the `api.db` file by default. You can find the file on the following locations depending on your operating system.
| Platform | Value                                                            | Example                                                          |
| -------- | ---------------------------------------------------------------- | ---------------------------------------------------------------- |
| Linux    | `$XDG_DATA_HOME` or `$HOME`/.local/share/ai-health-assistant-api | /home/alice/.local/share/ai-health-assistant-api                 |
| macOS    | `$HOME`/Library/Application Support/ai-health-assistant-api      | /Users/Alice/Library/Application Support/ai-health-assistant-api |
| Windows  | `{FOLDERID_RoamingAppData}`\ai-health-assistant-api              | C:\Users\Alice\AppData\Roaming\ai-health-assistant-api           |
# Usage
This is an example of the output when running the application with the `--help` flag on a Linux machine. The output of the command will be different to fit the file system conventions of the host operating system. The command usage for the backend is the following:
```
The backend API for the chat application

Usage: ai-health-assistant-api [OPTIONS]

Options:
  -u, --db-url <DB_URL>  The URL of the database to connect to Will default to DATABASE_URL variable inside .env file if a .env file is found in the current project directory, otherwise `dirs::data_dir` if not provided [default: sqlite://{$XDG_DATA_HOME}/ai-health-assistant-api/api.db]
  -p, --port <PORT>      The port to listen on for connections [default: 3000]
  -d, --debug            Enable trace debugging for tokio-console
  -h, --help             Print help
  -V, --version          Print version

```
