function send_connection_error (res, message) {
	res.send(JSON.stringify({ error_message: message }));
}

function log_message (req, message) {
	console.log(`${new Date().toISOString()} | ${req.ip} | ${message}`);
}

module.exports = {
	err: send_connection_error,
	log: log_message
};
