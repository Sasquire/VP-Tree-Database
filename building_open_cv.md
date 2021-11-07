Clone https://github.com/opencv/opencv.git

Remove extra modules from opencv/platforms/js/opencv_js.config.py

This seems to be the minimum needed

```
core = {
    '': [],
    'Algorithm': [],
}

features2d = {
	'Feature2D': ['detectAndCompute'],
	'ORB': [
		'create',
		'setMaxFeatures',
		'setScaleFactor',
		'setNLevels',
		'setEdgeThreshold',
		'setFirstLevel',
		'setWTA_K',
		'setScoreType',
		'setPatchSize',
		'getFastThreshold',
		'getDefaultName'
	]
}

white_list = makeWhiteList([core, features2d])

```

docker run --rm -v $(pwd):/src -u $(id -u):$(id -g) emscripten/emsdk:2.0.10 emcmake python3 ./platforms/js/build_js.py build_js
