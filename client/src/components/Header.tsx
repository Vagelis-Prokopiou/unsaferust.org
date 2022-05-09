// https://tailwindcomponents.com/components/tables?page=2

import {JSX} from 'solid-js';

const Header: (props) => JSX.Element = (props) => {
    return (
        <nav
            class="dark:bg-gray-800 p-5">
            <div class="container flex flex-wrap items-center justify-between mx-auto">
                <a href="#" class="flex items-center">
                        <span
                            class="self-center text-xl font-semibold whitespace-nowrap dark:text-white">unsaferust.org</span>
                </a>

                {/* Mobile menu button */}
                <button data-collapse-toggle="mobile-menu" type="button"
                        class="inline-flex items-center justify-center ml-3 text-gray-400 rounded-lg md:hidden hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-300 dark:text-gray-400 dark:hover:text-white dark:focus:ring-gray-500"
                        aria-controls="mobile-menu-2" aria-expanded="false">
                    <span class="sr-only">Open main menu</span>
                    <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
                        <path fill-rule="evenodd"
                              d="M3 5a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 10a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 15a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z"
                              clip-rule="evenodd"></path>
                    </svg>
                    <svg class="hidden w-6 h-6" fill="currentColor" viewBox="0 0 20 20"
                         xmlns="http://www.w3.org/2000/svg">
                        <path fill-rule="evenodd"
                              d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                              clip-rule="evenodd">

                        </path>
                    </svg>
                </button>

                <div class="hidden w-full md:block md:w-auto" id="mobile-menu">
                    <ul class="flex flex-col mt-4 md:flex-row md:space-x-8 md:mt-0 md:text-sm md:font-medium">
                        <li>
                            <a onclick={
                                (e) => {
                                    e.preventDefault();
                                    props.navigate(props.ROUTE_HOME);
                                }
                            }
                               href="#"
                               class="dark:text-white dark:hover:text-gray-300 block py-2 pl-3 pr-4 md:p-0">Home</a>
                        </li>
                        <li>
                            <a onclick={
                                (e) => {
                                    e.preventDefault();
                                    props.navigate(props.ROUTE_LIST);
                                }
                            }
                               href="#"
                               class="dark:text-white dark:hover:text-gray-300 block py-2 pl-3 pr-4 md:p-0">Stats</a>
                        </li>
                        {/*<li>*/}
                        {/*    <form*/}
                        {/*        //action="/search"*/}
                        {/*        class="flex flex-wrap justify-between md:flex-row"><input type="search"*/}
                        {/*                                                                  name="query"*/}
                        {/*                                                                  placeholder="Search"*/}
                        {/*                                                                  class="w-full h-12 px-4 text-sm text-gray-700 bg-white border border-gray-200 rounded-lg lg:w-20 xl:transition-all xl:duration-300 xl:w-36 xl:focus:w-44 lg:h-10 dark:bg-gray-900 dark:text-gray-300 dark:border-gray-600 focus:border-primary dark:focus:border-primary focus:outline-none focus:ring focus:ring-primary dark:placeholder-gray-400 focus:ring-opacity-20"/>*/}
                        {/*    </form>*/}
                        {/*</li>*/}
                    </ul>
                </div>
            </div>
        </nav>
    );
};
export default Header;
