import json
import uuid
import argparse
import base64
from datetime import datetime
from cassandra.cluster import Cluster
from cassandra.auth import PlainTextAuthProvider

parser = argparse.ArgumentParser(description="Insert HTTP target")
parser.add_argument("--uname", required=True, help="Username")
parser.add_argument("--pass", required=True, help="Password")
parser.add_argument("--name", required=True, help="Name")
parser.add_argument("--target", required=True, help="HTTP or HTTPs URI")
parser.add_argument("--method", required=True, default="GET", help="HTTP method")
parser.add_argument("--interval", required=True, type=int, help="Interval in seconds")
parser.add_argument("--success-min", required=False, default=200, type=int, help="Minimum successful HTTP status, default is 200")
parser.add_argument("--success-max", required=False, default=299, type=int, help="Maximum successful HTTP status, default is 299")
parser.add_argument("--insecure", required=False, default=False, type=bool, help="Allow insecure connection")
parser.add_argument("--redirect", required=False, type=bool, help="Follow redirects")
parser.add_argument("--redirect-max", required=False, default=5, type=int, help="Maximum redirects, by default is 5 and only enabled with --redirect")
parser.add_argument("--body", required=False, help="The body of the request")
parser.add_argument("--header", required=False, action='append', help="Header, must be {HEADER NAME}={VALUE}")
parser.add_argument("--timeout", required=False, type=int, help="Request timeout")

args = parser.parse_args()
metadata = {};

metadata["m"] = args.method
metadata["mi"] = args.success_min
metadata["mx"] = args.success_max
metadata["i"] = args.insecure

if args.redirect is not None:
    metadata["r"] = args.redirect_max

if args.redirect is not None:
    metadata["b"] = base64.b64encode(args.body.encode('utf-8')).decode('utf-8')

if args.header is not None:
    headers = {}

    for header in args.header:
        key, value = header.split('=', 1)
        headers[key.strip()] = value.strip()

    metadata["h"] = headers

if args.timeout is not None:
    metadata["t"] = args.timeout

current_time = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
cluster = Cluster(['localhost'])
session = cluster.connect('upchime')

cql = """
INSERT INTO target (
    target_id,
    target_enabled,
    target_name,
    target_address,
    target_ping_type,
    target_interval,
    target_state,
    target_created_at,
    target_updated_at,
    target_metadata
) VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
"""
try:
    session.execute(cql, (
        uuid.uuid4(),  # Let Python generate UUID
        True,
        args.name,
        args.target,
        'HTTP',
        args.interval,
        0,
        current_time,
        current_time,
        json.dumps(metadata)
    ))
    print("inserted")
except Exception as e:
    print(f"error: {e}")
finally:
    cluster.shutdown()
