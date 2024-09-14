pub static HTMLIndexData: &str = r#"
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RSS Translate</title>
    <script src="https://code.jquery.com/jquery-3.7.1.min.js" integrity="sha256-/JqT3SQfawRcv/BIHPThkBvs0OEvtFFmqPF/lYI/Cxo=" crossorigin="anonymous"></script>
    <script src="https://cdn.tailwindcss.com"></script>
    <style>
        body {
            font-family: Arial, sans-serif;
        }
    </style>
</head>
<body class="bg-gray-100 p-8">
    <div class="max-w-md mx-auto bg-white rounded-xl shadow-md overflow-hidden md:max-w-2xl p-6">
        <h1 class="text-2xl font-bold mb-4">RSS Translate</h1>
        <form id="translateForm" class="space-y-4">
            <div>
                <label for="url" class="block text-sm font-medium text-gray-700">URLを入力</label>
                <input type="url" id="url" name="url" required class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50 px-2 py-3">
            </div>
            <div>
                <label for="lang" class="block text-sm font-medium text-gray-700">言語を選択</label>
                <select id="lang" name="lang" class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50  px-2 py-3">
                    <option value="ja-JP" selected>日本語 (日本)</option>
                    <option value="en-US">English (United States)</option>
                    <option value="en-GB">English (United Kingdom)</option>
                    <option value="es-ES">Español (España)</option>
                    <option value="fr-FR">Français (France)</option>
                    <option value="de-DE">Deutsch (Deutschland)</option>
                </select>
            </div>
            <button type="submit" class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 px-2 py-3">変換</button>
        </form>
        <div id="result" class="mt-4 hidden">
            <h2 class="text-lg font-semibold mb-2">変換結果:</h2>
            <p id="convertedUrl" class="text-sm text-gray-600 break-all"></p>
            <button id="copyButton" class="mt-2 py-1 px-3 bg-gray-200 text-sm rounded hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-400">コピー</button>
        </div>
    </div>

    <script>
        $(document).ready(function() {
            $('#translateForm').on('submit', function(e) {
                e.preventDefault();
                const inputUrl = encodeURIComponent($('#url').val());
                const selectedLang = $('#lang').val();
                const convertedUrl = `https://rss-translate.mox.run/rss?url=${inputUrl}&lang=${selectedLang}`;

                $('#convertedUrl').text(convertedUrl);
                $('#result').removeClass('hidden');

                // Copy to clipboard
                navigator.clipboard.writeText(convertedUrl).then(function() {
                    console.log('URL copied to clipboard');
                }).catch(function(err) {
                    console.error('Failed to copy URL: ', err);
                });
            });

            $('#copyButton').on('click', function() {
                const convertedUrl = $('#convertedUrl').text();
                navigator.clipboard.writeText(convertedUrl).then(function() {
                    console.log('URL copied to clipboard');
                    alert('URLがクリップボードにコピーされました');
                }).catch(function(err) {
                    console.error('Failed to copy URL: ', err);
                    alert('URLのコピーに失敗しました');
                });
            });
        });
    </script>
</body>
</html>

"#;
