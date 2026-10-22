# sectorbot
This is an aimless discord bot. Initially I am looking to implement a chat bot
in discord that is just responsible for posting new tweets from select twitter
account to a discord channel.

# Design and Deployment
I want to deploy in a scratch docker container so this prohect needs to be
statically linked. In order to do this in rust, we have to use a different libc
runtime called musl. The main elements that cannot be used are openssl. So we 
choose to force the use of `rustls`.