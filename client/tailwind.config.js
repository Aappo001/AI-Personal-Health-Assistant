/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        "main-black": "#0e100f",
        "offwhite": "#fffce1",
        "main-green": "#0ae448",
        "lilac": "#9d95ff",
        "lt-green": "#abff84",
        "orangey": "#ff8709",
        "main-pink": "#fec5fb",
        "shock-pink": "#f100cb",
        "main-blue": "#00bae2",
        "surface75": "#bbbaa6",
        "surface50": "#7c7c6f",
        "surface25": "#42433d",
        "main-grey": "#676767"

      },
    },


  },
  plugins: [],
}

// --gray-400: #b4b4b4;
//   --gray-500: #9b9b9b;
//   --gray-600: #676767;
//   --gray-700: #424242;
//   --gray-750: #2f2f2f;
//   --gray-800: #212121;
//   --gray-900: #171717;
//   --gray-950: #0d0d0d;

