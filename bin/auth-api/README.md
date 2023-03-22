# SI Auth API

### Prisma / DB

Use `pnpx prisma` to run prisma commands locally. For example
- `pnpx prisma migrate dev --name something-descriptive` - generates and runs new migration based on prisma schema
- `pnpx prisma migrate reset` - wipes db, re-runs all migrations
- `pnpx prisma db push` - push changes directly to db without any migrations (good for experimentation)

### JWT Signing Key
- `ssh-keygen -t rsa -b 4096 -m PEM -f jwtRS256.key`
- `openssl rsa -in jwtRS256.key -pubout -outform PEM -out jwtRS256.key.pub`


### Config / env vars
- Config is loaded from .env files
- Put local overrides in gitignored .env.local
- on deployed environments override using actual env vars
- auto-restart is not currently triggered on .env file changes (see [issue](https://github.com/nodejs/node/issues/45467))


## Running auth stack locally
By default, our system is set up to hit the production auth stack because we still want real auth when running local dev instances.

While working on the auth stack, we still need to run it locally and configure things to point to our local auth stack:
- point our web app at the local auth stack by adding this to `app/web/.env.local`
  ```
    VITE_AUTH_API_URL=http://localhost:9001
    VITE_AUTH_PORTAL_URL=http://localhost:9000
  ```
- point sdf to the auth api (TODO: figure out best way to pull from env)
- (OPTIONAL) set auth-api redis url to a local redis instance in .env.local (ex: `REDIS_URL=127.0.0.1:6379`) only if needing to test redis. Falls back to in-memory storage...
- run the auth api `pnmp run dev` in this directory or `pnpm dev:auth-api` at the root
- run the auth portal `pnmp run dev` in `app/auth-portal` or `pnpm dev:auth-portal` at the root
- (or run both by running `pnpm run dev:auth` at the root)
