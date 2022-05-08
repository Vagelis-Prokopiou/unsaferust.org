// https://tailwindcomponents.com/components/tables?page=2

import {JSX, Show} from 'solid-js';

const Header: (props) => JSX.Element = (props) => {
    return (
        <header
            class="bg-white border-b dark:bg-gray-900 dark:border-gray-700 lg:fixed lg:w-full lg:top-0 lg:left-0 lg:z-30">
            <div
                class="container px-4 py-5 mx-auto space-y-4 lg:space-y-0 lg:flex lg:items-center lg:justify-between lg:space-x-10">


                <div class="flex justify-between">
                    <div class="flex items-center">
                        {/*<svg fill="none" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 25 27"*/}
                        {/*     class="h-7 w-7 -mt-2 flex-shrink-0">*/}
                        {/*    <path*/}
                        {/*        d="M22.487.658s5.03 13.072-1.822 22.171C16.476 28.39 9.84 27.26 5.484 25.68c1.513-3.391 3.441-6.067 5.784-8.03 1.176.623 3.186.792 6.03.51-2.535-.221-4.284-.654-5.246-1.3l.125.08c2.122-1.546 4.556-2.556 7.303-3.029-3.16-.285-6.026.315-8.598 1.804-.577-.742-1.157-1.748-1.74-3.018.07 1.534.339 2.734.809 3.6-2.64 1.797-4.953 4.58-6.94 8.351a7.583 7.583 0 01-.188-.088c-.802-.358-1.328-1.037-1.755-2.036C-1.9 13.366 4.645 8.273 11.123 7.989 23.046 7.465 22.487.658 22.487.658z"*/}
                        {/*        fill="#0ED3CF" data-darkreader-inline-fill=""*/}
                        {/*        style="--darkreader-inline-fill:#3bf2ee;"></path>*/}
                        {/*</svg>*/}
                        <p class="text-gray-800 dark:text-gray-200 text-xl ml-2"><strong>unsaferust.org</strong></p>
                    </div>
                </div>


                <div class="hidden lg:flex lg:flex-row lg:items-center lg:justify-between lg:flex-1 lg:space-x-2">
                    <div
                        class="flex flex-col space-y-3 lg:space-y-0 lg:flex-row lg:space-x-6 xl:space-x-8 lg:items-center">
                        {/*<a href="/"*/}
                        {/*   class="text-gray-500 dark:text-gray-200 hover:text-gray-800 dark:hover:text-gray-400 transition-colors duration-300">Home</a>*/}
                    </div>


                    <div class="flex flex-col space-y-4 lg:space-y-0 lg:flex-row lg:items-center lg:space-x-4">
                        <form
                            //action="/search"
                            class="flex flex-wrap justify-between md:flex-row"><input type="search"
                                                                                      name="query"
                                                                                      placeholder="Search"
                                                                                      class="w-full h-12 px-4 text-sm text-gray-700 bg-white border border-gray-200 rounded-lg lg:w-20 xl:transition-all xl:duration-300 xl:w-36 xl:focus:w-44 lg:h-10 dark:bg-gray-900 dark:text-gray-300 dark:border-gray-600 focus:border-primary dark:focus:border-primary focus:outline-none focus:ring focus:ring-primary dark:placeholder-gray-400 focus:ring-opacity-20"/>
                        </form>

                        <ul class="flex flex-col mt-4 md:flex-row md:space-x-8 md:mt-0 md:text-sm md:font-medium">
                            <li>
                                <a target="_blank" href="https://github.com/Vagelis-Prokopiou/unsaferust.org"
                                   class="block py-2 pl-3 pr-4 text-white bg-blue-700 rounded md:bg-transparent md:text-blue-700 md:p-0 md:dark:text-white dark:bg-blue-600 md:dark:bg-transparent"
                                   aria-current="page">
                                    GitHub
                                </a>
                            </li>
                            <li>
                                <a href="#"
                                   class="block py-2 pl-3 pr-4 text-gray-700 border-b border-gray-100 hover:bg-gray-50 md:hover:bg-transparent md:border-0 md:hover:text-blue-700 md:p-0 dark:text-gray-400 dark:hover:text-white dark:border-gray-700 dark:hover:bg-gray-700 md:dark:hover:bg-transparent">
                                    Contact
                                </a>
                            </li>
                        </ul>
                    </div>
                </div>
            </div>
        </header>
    );
};

export default Header;
