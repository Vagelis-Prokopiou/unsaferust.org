import {JSX} from 'solid-js';

const Pager: (props) => JSX.Element = (props) => {
    return (
        <div>
            <div class="flex flex-col items-center">
                <span class="text-sm text-gray-700 dark:text-gray-400">
                Showing <span
                    class="font-semibold text-gray-900 dark:text-white"> {props.numberOfShownRecords()} </span> of <span
                    class="font-semibold text-gray-900 dark:text-white"> {props.numberOfTotalItems} </span> Entries
                </span>
                <div class="inline-flex mt-2 xs:mt-0">
                    <button onclick={() => props.paginate("desc")}
                            class="inline-flex items-center py-2 px-4 text-sm font-medium hover:bg-gray-100 rborder-gray-500 rounded-r border-l hover:bg-gray-900 dark:bg-gray-800 dark:border-gray-700 dark:text-gray-400 dark:hover:bg-gray-700">
                        <svg class="mr-2 w-5 h-5" fill="currentColor" viewBox="0 0 20 20"
                             xmlns="http://www.w3.org/2000/svg">
                            <path fill-rule="evenodd"
                                  d="M7.707 14.707a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l2.293 2.293a1 1 0 010 1.414z"
                                  clip-rule="evenodd"></path>
                        </svg>
                        Prev
                    </button>
                    <button onclick={() => props.paginate("asc")}
                            class="inline-flex items-center py-2 px-4 text-sm font-medium hover:bg-gray-100 rborder-gray-500 rounded-r border-l hover:bg-gray-900 dark:bg-gray-800 dark:border-gray-700 dark:text-gray-400 dark:hover:bg-gray-700">
                        Next
                        <svg class="ml-2 w-5 h-5" fill="currentColor" viewBox="0 0 20 20"
                             xmlns="http://www.w3.org/2000/svg">
                            <path fill-rule="evenodd"
                                  d="M12.293 5.293a1 1 0 011.414 0l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414-1.414L14.586 11H3a1 1 0 110-2h11.586l-2.293-2.293a1 1 0 010-1.414z"
                                  clip-rule="evenodd"></path>
                        </svg>
                    </button>
                </div>
            </div>
        </div>
    );
};

export default Pager;
