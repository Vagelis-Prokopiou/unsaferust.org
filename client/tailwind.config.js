module.exports = {
    darkMode: 'class', // See https://tailwindcss.com/docs/dark-mode#basic-usage
    purge: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
    content: [],
    theme: {
        extend: {},
    },
    plugins: [
        require('flowbite/plugin') // See https://flowbite.com/docs/getting-started/quickstart/
    ]
}
