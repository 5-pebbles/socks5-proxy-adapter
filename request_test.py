import requests

# Set up the SOCKS5 proxy
proxies = {
    'http': 'socks5://127.0.0.1:6969',
    'https': 'socks5://127.0.0.1:6969',
}

# The URL you want to access
url = 'https://ipwho.is'

try:
    # Send a GET request through the SOCKS5 proxy
    response = requests.get(url, proxies=proxies)

    # Check if the request was successful
    if response.status_code == 200:
        # Print the content of the response
        print(response.text)
    else:
        print(f"Request failed with status code: {response.status_code}")
except requests.exceptions.RequestException as e:
    print(f"Request error: {e}")