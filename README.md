# tinybase

## What is this?
This project is a simple REST API that mimics the same endpoints as [Replit DB's](https://docs.replit.com/hosting/database-faq). With it being the same, you can use any of the community's clients, which will be plug-and-play with this database! However, this database does have one twist it can send WebSocket updates! So you can use this to listen for any changes to a key prefix! An example will be if you have a chat application and a key of `messages:room_name:message_id`. Then, if someone sends a new message and you save it in `messages:room_name:*`, it will update all the WebSockets subscribed!

## But, why?
I have wanted to learn rust, and I recently created a project with [supabase](https://supabase.com/) and thought it would be fun to make a tiny version of it and implement it with rust mimicking Replits DB so that others can use it! This project will release a new npm package for you to use alongside @replit/database and an example project on how to use it! The goal was for me to learn rust and create a simple-to-use interface as Replit did with their database so new developers can pick it up and use it easily in their projects!

## Limitations
* I would not use this in a production environment, but as for a hobby project i think this will work well!
* It is expected for you to run this on the same Replit, or in a new one.


## Wip usage
Write about how to set up with secret and have directions for in a replit with secrets

## WIP binary usage
Look into setting this DB up as a binrary to run on a replit. Can probably call local host with it on replit

## WIP Developer setup
1. Set db .env file
2. Install diesel cli
3. run `diesel setup`