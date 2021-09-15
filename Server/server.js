const express = require('express');
const formData = require('express-form-data');
const bodyParser = require('body-parser');
const fs = require('fs');
const path = require('path');
const settings = require('./../secret.json');
const database_interface = require('./database_interface.js');
const { log } = require('./shared.js');

const app = express();
app.use(formData.parse());
app.use(bodyParser.json());

app.get('/', (req, res) => {
	log(req, 'Client connected');
	res.send(fs.readFileSync(path.join(__dirname, 'UI', 'index.html'), 'utf8'));
});

app.post('/get_image_results.json', async (req, res) => {
	log(req, 'Requesting vectors from an image');
	try {
		const [file_path, k] = await database_interface.verify_input(req.files, req.body.k, req, res);
 		const output = await database_interface.run_program(file_path, k, req, res);
		res.send(output);
	} catch (e) {
		log(req, e);
	}
});

app.listen(settings.port, () => {
	console.log(`Example app listening at http://localhost:${settings.port}`);
});
