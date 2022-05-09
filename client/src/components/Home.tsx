import {JSX} from "solid-js";

const Home: (props) => JSX.Element = (props) => {
    // console.log('import.meta.env', import.meta.env)
    return (
        <div>
            <h4 class="text-5xl font-semibold text-center text-gray-800 dark:text-gray-200">
                Welcome to unsaferust.org
            </h4>

            <p class="dark:text-white text-xl pt-10 pb-3">
                The goal of this site/project, is to provide information related to the usage of the "unsafe" keyword
                within the source code of a given crate.
            </p>
            <p class="dark:text-white text-xl pb-3">
                It started as a personal project. My initial incentive was to gather this information for me,
                in order to be able to make informed decisions on crates that I would prefer using in my projects,
                or, at least, be aware of what the crates that I use do, regarding this specific issue.
            </p>
            <p class="dark:text-white text-xl pb-3">
                After thinking about it, it dawn on me that probably others are also interested on this subject,
                so I decided to put something together and publish it. This is the (non-finished) result (a number of
                Todos are pending).
            </p>
            <p class="dark:text-white text-xl pb-3">
                Through this project, I am, in no way, trying to bash on crates that make heavy use of "unsafe" code.
                Neither do I intend to provoke "flame wars" in any way, shape or form.
                I am just trying to provide something useful to the Rust community, and raise awareness regarding the
                use of "unsafe" code (see <a target="_blank"
                                             href="https://cve.mitre.org/cgi-bin/cvekey.cgi?keyword=Rust">Rust CVEs</a>).
            </p>
            <p class="dark:text-white text-xl pb-3">
                Any kind of collaboration can be done through <a target="_blank" class=""
                                                                 href="https://github.com/Vagelis-Prokopiou/unsaferust.org">GitHub</a>.
            </p>
            <p class="dark:text-white text-xl pb-3">
                Enjoy.
            </p>
        </div>
    );
};

export default Home;