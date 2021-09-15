const path = require('path');
const { err, log } = require('./shared.js');

async function verify_input (files, k, req, res) {
	if (Object.keys(files).length > 1) {
		log(req, 'Multiple files submitted');
		err(res, 'Multiple files submitted');
		throw new Error('Multiple files submitted');
	} else if (Object.keys(files).length === 0) {
		log(req, 'No files submitted');
		err(res, 'No files submitted');
		throw new Error('Multiple files submitted');
	}

	try {
		const tmp_image_path = files.file0.path;
		const verified_path = path.parse(tmp_image_path);
		const full_path = path.join(verified_path.dir, verified_path.base);

		const image_result = await spawn('file', ['-bi', full_path], {});
		const valid_types = ['image/png; charset=binary', 'image/jpeg; charset=binary'];
		if (valid_types.includes(image_result.trim()) === false) {
			throw new Error();
		}

		return [full_path, parseInt(k, 10)];
	} catch (e) {
		log(req, `Error validating path '${files.file0.path}'`);
		err(res, 'Error validating path');
		throw new Error(`Error validating path '${files.file0.path}'`);
	}
}

async function run_program (image_path, k, req, res) {
	log(req, `Extracting data from ${image_path}`);
	const program_path = path.join(__dirname, '..', 'VP-Tree-Database', 'target', 'release', 'feature_database');
	const args = ['-k', k.toString(), '-f', image_path];
	const options = {
		cwd: path.join(__dirname, '..', 'VP-Tree-Database')
	};
	const output_string = await spawn(program_path, args, options)
		.catch(stderr => {
			log(req, `Error extracting key points\n${stderr.toString()}\n`);
			err(res, 'Error extracting key points');
			throw new Error(`Error extracting key points\n${stderr.toString()}\n`);
		});
	log(req, `Data extracted from ${image_path}`);

	try {
		log(req, `Parsing data from ${image_path}`);
		let result = parse_output(output_string);
		log(req, `Data parsed from ${image_path}`);
		return result;
	} catch (e) {
		log(req, `Error parsing key point data\n${e}\n`);
		err(res, 'Error parsing key point data');
		throw new Error(`Error parsing key point data\n${e}\n`);
	}
}

function parse_output (output_string) {
	return output_string
		.split('\n')
		.filter(e => e)
		.filter(e => e.startsWith('input') === false)
		.map(e => e.split(' ').filter(e => e))
		.map(parse_vector);
}

function parse_vector (output_array) {
	if (output_array.length === 13) {
		return parse_header(output_array);
	} else {
		return parse_data(output_array);
	}

	function parse_header (e) {
		return {
			input_feature: parseInt(e[0], 10),
			comparisons: parseInt(e[5], 10),
			x: parseFloat(e[7]),
			y: parseFloat(e[8]),
			size: parseFloat(e[9]),
			angle: parseFloat(e[10]),
			response: parseFloat(e[11]),
			octave: parseInt(e[12], 10),
		};
	}

	function parse_data (e) {
		return {
			input_feature: parseInt(e[0], 10),
			rank: parseInt(e[1], 10),
			distance: parseInt(e[2], 10),
			md5: e[3],
			file_ext: e[4],
			frame: parseInt(e[5], 10),

			x: parseFloat(e[8]),
			y: parseFloat(e[9]),
			size: parseFloat(e[10]),
			angle: parseFloat(e[11]),
			response: parseFloat(e[12]),
			octave: parseInt(e[13], 10),
		}
	}
}

async function spawn(command, args, options) {
	const { spawn } = require('child_process');
	const child = spawn(command, args, options);

	let data_string = "";
	for await (const chunk of child.stdout) {
		data_string += chunk;
	}
	
	let error_string = "";
	for await (const chunk of child.stderr) {
		error_string += chunk;
	}

	const exit_code = await new Promise((resolve, reject) => {
		child.on('close', resolve);
	});

	if (error_string.length !== 0) {
		throw new Error(error_string);
	} else {
		return data_string;
	}
}

module.exports = {
	verify_input: verify_input,
	run_program: run_program
};
