#!/usr/bin/env python
# a python script to get the tip header from CKB rpc
# curl -s -X POST 127.0.0.1:18200  -H 'Content-Type: application/json' -d '{"jsonrpc": "2.0", "method": "get_peers", "params": [], "id": "1"}' 


# the first argument is url; then request the server
import sys
import requests
import json

url = sys.argv[1]
payload = {"jsonrpc": "2.0", "method": "get_tip_header", "params": [], "id": "1"}
headers = {'Content-Type': 'application/json'}
response = requests.request("POST", url, headers=headers, data=json.dumps(payload)).json()


# the "result.number", "result.epoch" and  "result.timestamp" fields in response is a hex encoded block number, convert it to decimal
# make a object from the reponse, and convert the hex encoded fields into decimal and print it

response['result']['number'] = str(int(response['result']['number'], 16))
response['result']['timestamp'] = str(int(response['result']['timestamp'], 16))

# the response['result']['timestamp'] is unixtimestamp * 1000, convert it  to a human readable time with UTC+0 annotation
import datetime
response['result']['timestamp'] = datetime.datetime.utcfromtimestamp(int(response['result']['timestamp'])/1000).strftime('%Y-%m-%d %H:%M:%S') + ' UTC+0'

# pretty print response with color
result=json.dumps(response, indent=4, sort_keys=True)

# result is a json string, print it with color
import pygments
from pygments.lexers import JsonLexer
from pygments.formatters import TerminalFormatter
print(pygments.highlight(result, JsonLexer(), TerminalFormatter()))


