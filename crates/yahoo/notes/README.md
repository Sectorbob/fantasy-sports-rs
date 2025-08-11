# We are doing the oauth2 

https://developer.yahoo.com/oauth2/guide/flows_authcode/

Basically omitting client_secret, and using the S256 code verifier challenge thing

## Client Side App Use Case
There isn't much documentation online around how to properly do a client side use case (no CLIENT_SECRET). The scripts in this folder show how to do this with a bash script and curl.

```shell
./oauth_request_auth
```
