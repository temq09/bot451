This is a telegram bot to download web pages via telegram bot to bypass pages blocked with 451 error code.

Note: the architecture might look too complicated for this small project, but it was done on purpose to lern on how to split components to make it scalable. 


## Overview
The bot consists of two parts: the actual bot that handles telegram commands and the REST backend that is responsible for downloading the page and sending it back to the user.
The bot part can be ran in the standalone mode so it does not require the backend to handle requests.

Running the bot in the distributed mode with a separate backend can help to handle a big number of requests as it can be run on a more powerful server and can be scaled horizontally.

## Bot deployment

Before starting the bot follow the [steps](https://core.telegram.org/bots/tutorial) to create your bot in telegram and get a token for it.
Add the token as the `TELOXIDE_TOKEN` environmental variable.

There is a docker image that provides all dependencies, but it can also be run from a binary.

### Binary

When running in the standalone mode it's required to provide a path to [singilefile](https://github.com/gildas-lormeau/SingleFile) binary.

Supported parameters:
- `backend_url` - the url for the backend to serve the requests, required for the distributed mode
- `singlefile_cli` - path to the singlefile binary, required for standalone mode
- `work_dir` - path to the folder needed to save the pages, required for the standalone mode
- `throttling_timeout_seconds` - throttling interval for requests from the same client

To start the bot in the standalone mode:
```bash
./bot --singlefile_cli=<path_to_binary> --work_dir=<path>
```

To start the bot in the distributed mode:
```bash
./bot --backend_url=example.com
```

### Dockerimage

For the simplicity there is a docker image with all dependencies preinstalled.

```bash
docker build -f Dockerfile.bot -t bot .
docker container run bot <params>
```

### Backend deployment
The bot can be run with a separate backend to server the requests.

The recomended way to run the backend is docker.

The beckend requires a database to work. There are two options for the database: `sqlite` or `postgres`.

The sqlite option is built-in into the docker and will be used out of the box if postgres endpoint is not found.
It also requires singlefile binary which comes preinstalled with the docker image.

#### Supported parameters:
- `pg_url` - Postgres endpoint, will be run with sqlite if omitted 
- `work_dir` - a directory to download files and store sqlite database, required
- `pg_user` - database user, required when `pg_url` is set
- `pg_password` - password for the user, required when `pg_url` is set 
- `pg_database` - database name in postgres deployment, required when `pg_url` is set
- `singlefile_cli` - path to the singlefile binary

```bash
docker build -f Dockerfile.backend -t backend .
docker container run backend <params>
```

## Known limitation/issues
- Caching does not work properly with pages that have ads built-in. Every time a page loads a new adds usually appears which breaks comparison check. A possible solution could be running an ads blocker on the host that loads pages
- Accept cookies popup is visible and could block content without an option to close it

## Future plans
- Implement a browser extension that could open up bot's chat in the telegram app with url copied to the clipboard of the link inserted to the message box.
