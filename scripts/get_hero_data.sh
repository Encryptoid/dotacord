#!/bin/bash
curl https://api.opendota.com/api/heroes | jq > ./data/heroes.json 
