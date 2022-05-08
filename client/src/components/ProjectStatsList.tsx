// Flowbite components
// https://flowbite.com/docs/components/pagination/#table-data-pagination-with-icons

import {For, JSX, Show} from 'solid-js';


const ProjectStatsList: (props) => JSX.Element = (props) => {
    // console.log('import.meta.env', import.meta.env)
    return (
        <div>
            <h4 class="text-5xl font-semibold text-center text-gray-800 dark:text-gray-200">
            Current projects stats
            </h4>

            <div class="flex flex-col">
                <div class="overflow-x-auto sm:-mx-6 lg:-mx-8">
                    <div class="py-4 inline-block min-w-full sm:px-6 lg:px-8">
                        <div class="overflow-hidden">
                            <table class="min-w-full text-center">
                                <thead class="border-b bg-gray-800">
                                <tr>
                                    <th scope="col" class="text-white px-6 py-4">Name</th>
                                    <th scope="col" class="text-white px-6 py-4">Code lines</th>
                                    <th scope="col" class="text-white px-6 py-4">Unsafe lines</th>
                                    <th scope="col" class="text-white px-6 py-4">Created at</th>
                                    <th scope="col" class="text-white px-6 py-4">Updated at</th>
                                    <th scope="col" class="text-white px-6 py-4">Project details
                                    </th>
                                    <th scope="col" class="text-white px-6 py-4">
                                        <label for="items-per-page"
                                               class="block mb-2 text-sm font-medium text-gray-900 dark:text-gray-400"
                                        >Select items per page</label>
                                        <select
                                            id="items-per-page"
                                            onInput={(e) => {
                                                const limit = e.target.value;
                                                const options = {...props.getPaginationOptions()};
                                                options.limit = limit;
                                                props.setPaginationOptions(options);
                                            }}
                                            class="bg-gray-50 border border-gray-300 text-gray-900 mb-6 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500">
                                            <option value="25">25</option>
                                            <option value="50">50</option>
                                            <option value="100">100</option>
                                            <option value="500">500</option>
                                        </select>
                                    </th>
                                </tr>
                                </thead>
                                <Show when={props.data.length}>
                                    <tbody>
                                    <For each={props.data}>
                                        {
                                            (projectStat) => {
                                                return (
                                                    <tr>
                                                        <td class="text-white px-6 py-4">
                                                            <a target="_blank"
                                                               href={projectStat.url}>{projectStat.name}</a>
                                                        </td>
                                                        <td class="text-white px-6 py-4">{projectStat.code_lines}</td>
                                                        <td class="text-white px-6 py-4">{projectStat.unsafe_lines}</td>
                                                        <td class="text-white px-6 py-4">{projectStat.created_at}</td>
                                                        <td class="text-white px-6 py-4">{projectStat.updated_at}</td>
                                                        <td class="text-white px-6 py-4">
                                                            <button
                                                                onclick={() => props.navigate(projectStat.project_id)}
                                                                type="button"
                                                                class="text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm p-2.5 text-center inline-flex items-center mr-2 dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800">
                                                                <svg class="w-5 h-5" fill="currentColor"
                                                                     viewBox="0 0 20 20"
                                                                     xmlns="http://www.w3.org/2000/svg">
                                                                    <path fill-rule="evenodd"
                                                                          d="M10.293 3.293a1 1 0 011.414 0l6 6a1 1 0 010 1.414l-6 6a1 1 0 01-1.414-1.414L14.586 11H3a1 1 0 110-2h11.586l-4.293-4.293a1 1 0 010-1.414z"
                                                                          clip-rule="evenodd"></path>
                                                                </svg>
                                                            </button>
                                                        </td>
                                                    </tr>
                                                );
                                            }
                                        }
                                    </For>
                                    </tbody>
                                </Show>
                            </table>
                        </div>
                    </div>
                </div>
            </div>


        </div>
    );
};

export default ProjectStatsList;
