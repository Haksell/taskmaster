import http.server
import socketserver

# Define the port to listen on
port = 8844
messages = []


class MyHandler(http.server.SimpleHTTPRequestHandler):
    def do_POST(self):
        content_length = int(self.headers['Content-Length'])
        post_data = self.rfile.read(content_length).decode('utf-8')

        messages.append(post_data)

        self.send_response(200)
        self.end_headers()

    def log_message(self, format, *args):
        pass

    def do_GET(self):
        # Generate an HTML page to display the messages and add auto-refresh
        self.send_response(200)
        self.send_header('Content-type', 'text/html')
        self.end_headers()

        # JavaScript to refresh the page every 5 seconds
        refresh_script = """
        <script>
            setTimeout(function(){
                location.reload();
            }, 1000);
        </script>
        """

        message_list = "<pre style='font-family: monospace;'>"
        for message in messages:
            message_list += message + "\n"
        message_list += "</pre>"

        html_page = f"""
        <!DOCTYPE html>
        <html>
        <head>
            <title>Message Log</title>
        </head>
        <body>
            <h1>Messages</h1>
            {message_list}
            {refresh_script}
        </body>
        </html>
        """

        self.wfile.write(html_page.encode('utf-8'))


if __name__ == '__main__':
    with socketserver.TCPServer(('0.0.0.0', port), MyHandler) as httpd:
        print(f"Serving on port {port}")
        httpd.allow_reuse_address = True
        httpd.serve_forever()
